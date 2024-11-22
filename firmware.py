import struct

import network
import socket
import time

import usb.device
from usb.device.mouse import MouseInterface

# For connecting to the Wi-Fi network
WIFI_SSID = ""
WIFI_PASSWORD = ""

# TCP server information
SERVER_PORT = 5000

# Define the struct constants for binary reading.
STRUCT_FORMAT = "<BiiB"

def connect_wlan():
    wlan0 = network.WLAN(network.STA_IF)

    # Disable any existing connections
    wlan0.active(False)
    wlan0.disconnect()

    # Connect to the Wi-Fi network
    wlan0.active(True)
    wlan0.connect(WIFI_SSID, WIFI_PASSWORD)

    # Wait until the connection is established
    print("Connecting to Wi-Fi network", end = "")
    while not wlan0.isconnected():
        time.sleep(1)
        print(".", end = "")
        pass

    print("\nConnected to Wi-Fi network")
    print("IP Address:", wlan0.ifconfig()[0])

    return wlan0

# Connect to the Wi-Fi network
wlan = connect_wlan()

# Create a TCP socket server
net = socket.socket()
net.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
net.bind(socket.getaddrinfo("0.0.0.0", SERVER_PORT)[0][-1])
net.listen(64)

# Create a mouse instance
mouse = MouseInterface()
usb.device.get().init(mouse, builtin_driver = True)

# Utility method for moving more than 128 units
def move_relative(x0, y0):
    rx = x0
    ry = y0

    while rx != 0 or ry != 0:
        dx = min(127, max(-127, rx))
        dy = min(127, max(-127, ry))

        mouse.move_by(dx, dy)

        rx -= dx
        ry -= dy

        time.sleep(0.02)

# Utility method for moving a group of points
GROUP_FORMAT = "<I"
SECTOR_FORMAT = "<ii"

# bytes0 starts with a byte, then 2 signed integers
def move_group(group_size, bytes0):
    pressed = False

    offset = 0
    for i in range(group_size):
        # Read the sector
        x0, y0 = struct.unpack(SECTOR_FORMAT, bytes0[offset:offset + 8])
        offset += 8

        # Move the mouse
        move_relative(x0, y0)

        if not pressed:
            time.sleep(0.01)
            pressed = True
            mouse.click_left(True)

        time.sleep(0.06)

    time.sleep(0.01)
    mouse.click_left(False)

def normalize():
    for i in range(10):
        mouse.move_by(-100, -100)
        time.sleep(0.01)

# Handles any incoming messages
def handle_message(opcode0, x0, y0, groups0, remaining):
    if opcode == 1: # Left click down
        mouse.click_left(True)
    elif opcode == 2: # Left click up
        mouse.click_left(False)
    elif opcode == 3: # Move mouse relative
        move_relative(x0, y0)
    elif opcode == 4: # Normalize to (0, 0)
        normalize()
    elif opcode == 5: # Moves the mouse with the provided array of points
        move_group(groups0, remaining)
    elif opcode == 6:
        normalize()
        time.sleep(0.01)
        move_relative(x0, y0)
    else:
        print("Invalid opcode:", opcode0)
        return

# Listen for messages
while True:
    try:
        # Try accepting a connection
        conn, addr = net.accept()

        # While the connection is alive, read messages
        while True:
            if not conn:
                break

            try:
                # Read any messages provided.
                message = conn.recv(512)

                # Destructure the message
                opcode, x, y, groups = struct.unpack(STRUCT_FORMAT, message[:10])

                # Handle the message accordingly
                handle_message(opcode, x, y, groups, message[10:])

                conn.send(b"OKAY")

            except Exception as ex:
                print("Failed to read message:", ex)
                break

        print("Connection closed")

        mouse.click_left(False)

    except Exception as ex:
        print("Failed to accept connection:", ex)