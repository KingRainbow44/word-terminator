use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpSocket, TcpStream};
use anyhow::{anyhow, Result};
use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::sleep;

pub enum Opcode {
    LeftDown,
    LeftUp,
    Move,
    Normalize,
    MoveGroup,
    NormalMove
}

impl Opcode {
    /// Converts the opcode into a byte.
    pub fn as_byte(&self) -> u8 {
        match self {
            Opcode::LeftDown => 1,
            Opcode::LeftUp => 2,
            Opcode::Move => 3,
            Opcode::Normalize => 4,
            Opcode::MoveGroup => 5,
            Opcode::NormalMove => 6
        }
    }
}

pub struct Instruction {
    opcode: Opcode,
    position: Option<(i32, i32)>,
    group: Option<Vec<(i32, i32)>>
}

impl Instruction {
    /// Creates a simple, empty instruction.
    /// opcode: The operation code.
    pub fn empty(opcode: Opcode) -> Self {
        Instruction { opcode, position: None, group: None }
    }

    /// Creates a mouse move instruction.
    /// dx: The change in the x-coordinate.
    /// dy: The change in the y-coordinate.
    pub fn delta(dx: i32, dy: i32) -> Self {
        Instruction { opcode: Opcode::Move, position: Some((dx, dy)), group: None }
    }

    /// Creates an absolute mouse move instruction.
    /// x: The x-coordinate to move to.
    /// y: The y-coordinate to move to.
    pub fn absolute(x: i32, y: i32) -> Self {
        Instruction { opcode: Opcode::NormalMove, position: Some((x, y)), group: None }
    }

    /// Creates a group move instruction.
    /// group: The group of positions to move to.
    pub fn group(group: &Vec<(i32, i32)>) -> Self {
        Instruction { opcode: Opcode::MoveGroup, position: None, group: Some(group.clone()) }
    }

    /// Serializes this instruction into binary.
    pub fn serialize(&self) -> Vec<u8> {
        // Read the position.
        let position = match self.position {
            Some((x, y)) => (x, y),
            None => (0, 0)
        };

        // Serialize the instruction.
        let mut bytes = BytesMut::new();
        bytes.put_u8(self.opcode.as_byte());
        bytes.put_i32_le(position.0);
        bytes.put_i32_le(position.1);

        // Serialize the group.
        if let Some(group) = &self.group {
            bytes.put_u8(group.len() as u8);

            for (x, y) in group {
                bytes.put_i32_le(*x);
                bytes.put_i32_le(*y);
            }
        } else {
            bytes.put_u8(0);
        }

        bytes.to_vec()
    }
}

pub struct Mouse {
    stream: TcpStream,

    // This is the current 'left mouse' button state.
    left: bool,

    // These instructions are used for exact movement.
    normalized: bool,
    current: (i32, i32)
}

impl Mouse {
    /// Creates a new networked mouse instance.
    /// hostname: The address of the mouse server.
    /// port: The port of the mouse server.
    pub async fn new<S: AsRef<str>>(hostname: S, port: u16) -> Result<Self> {
        // Parse the server address.
        let address = SocketAddr::new(hostname.as_ref().parse()?, port);

        // Connect to the server.
        let socket = TcpSocket::new_v4()?;
        let stream = socket.connect(address).await?;
        stream.set_nodelay(true)?;

        Ok(Mouse {
            stream,
            normalized: false,
            current: (0, 0),
            left: false
        })
    }

    /// Normalizes this mouse instance.
    /// This allows absolute movement to be used.
    pub async fn normalize(&mut self) {
        // Send the instruction.
        self.send(Instruction::empty(Opcode::Normalize)).await.unwrap();

        // Set the mouse to normalized mode.
        self.normalized = true;

        // Reset the current position.
        self.current = (0, 0);
    }

    /// Performs a single left click.
    pub async fn click(&mut self) {
        self.button(Some(true)).await;
        sleep(Duration::from_millis(50)).await;
        self.button(Some(false)).await;
    }

    /// Presses or releases the left mouse button.
    /// down: If true, the button is pressed. If false, the button is released.
    ///       When None, the button is toggled.
    pub async fn button(&mut self, down: Option<bool>) {
        // Get the 'left down' state.
        let new_state = match down {
            Some(state) => state,
            None => !self.left
        };
        self.left = new_state;

        // Send the instruction.
        let opcode = if new_state { Opcode::LeftDown } else { Opcode::LeftUp };
        self.send(Instruction::empty(opcode)).await.unwrap();
    }

    /// Moves the mouse relative to the current position.
    /// dx: The change in the x-coordinate.
    /// dy: The change in the y-coordinate.
    pub async fn move_relative(&mut self, dx: i32, dy: i32) -> Result<()> {
        // Send the instruction.
        self.send(Instruction::delta(dx, dy)).await?;

        // Update the current position.
        self.current.0 += dx;
        self.current.1 += dy;

        Ok(())
    }

    /// Moves the mouse relative using a list of points.
    /// UPDATE: This will also hold the left mouse button.
    /// group: The group of points to move to.
    pub async fn move_group(&mut self, group: Vec<(i32, i32)>) -> Result<()> {
        // Send the instruction.
        self.send(Instruction::group(&group)).await?;

        // Update the current position.
        self.current = group[group.len() - 1];

        Ok(())
    }

    /// Moves the mouse to the specified position.
    /// x: The x-coordinate to move to.
    /// y: The y-coordinate to move to.
    /// normal: Should the mouse be normalized before moving?
    pub async fn move_absolute(&mut self, position: (i32, i32), normal: bool) -> Result<()> {
        if !self.normalized && !normal {
            return Err(anyhow!("Mouse is not normalized."));
        }

        if !normal {
            // Calculate the change in position.
            let (x, y) = position;
            let dx = x - self.current.0;
            let dy = y - self.current.1;

            // Move the mouse relative to the current position.
            self.move_relative(dx, dy).await?;
        } else {
            self.normalized = true;
            
            // Send the instruction.
            self.send(Instruction::absolute(position.0, position.1)).await?;

            // Update the current position.
            self.current = position;
        }

        Ok(())
    }

    /// Moves the mouse to the specified position.
    /// instruction: The instruction to move the mouse.
    pub async fn send(&mut self, instruction: Instruction) -> Result<()> {
        // Serialize the instruction.
        let bytes = instruction.serialize();

        // Send the instruction.
        self.stream.write_all(&bytes).await?;

        // Wait for the server to reply.
        let mut reply = [0u8; 4];
        let size = self.stream.read(&mut reply).await?;

        if size != 4 {
            return Err(anyhow!("Failed to read the server reply."));
        }

        Ok(())
    }
}