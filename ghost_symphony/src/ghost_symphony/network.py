import subprocess
import logging
import sys

class NetworkGhost:
    """
    Manages network privacy features: MAC rotation, TTL hardening, and Kill-Switch.
    """

    def __init__(self):
        self.logger = logging.getLogger("NetworkGhost")

    def _run_command(self, cmd: list, description: str):
        """
        Helper to run system commands safely.
        """
        try:
            self.logger.info(f"Executing: {' '.join(cmd)}")
            subprocess.run(cmd, check=True, capture_output=True, text=True)
            self.logger.info(f"Successfully executed: {description}")
        except subprocess.CalledProcessError as e:
            self.logger.error(f"Failed to {description}: {e.stderr}")
            # We don't raise here to avoid crashing the whole app on minor network tweaks,
            # unless it's critical. For now, just log error.
        except FileNotFoundError:
             self.logger.error(f"Command not found: {cmd[0]}. Is the tool installed?")

    def rotate_mac(self, interface: str):
        """
        Randomizes the MAC address for the given interface using nmcli.
        """
        self.logger.info(f"Rotating MAC address for {interface}...")
        # nmcli device modify <iface> 802-11-wireless.cloned-mac-address random
        # Note: This changes the connection profile active on the device.
        # Ideally, we should modify the connection, but the brief said:
        # "nmcli device modify..."
        cmd = ["nmcli", "device", "modify", interface, "802-11-wireless.cloned-mac-address", "random"]
        self._run_command(cmd, "randomize MAC address")

    def harden_ttl(self):
        """
        Sets the default TTL to 128 (Windows-like) to blend in.
        """
        self.logger.info("Hardening TTL...")
        cmd = ["sysctl", "-w", "net.ipv4.ip_default_ttl=128"]
        self._run_command(cmd, "set default TTL to 128")

    def panic_shutdown(self):
        """
        Kill-Switch: Immediately disables networking to prevent leaks.
        """
        self.logger.critical("INITIATING PANIC SHUTDOWN - KILLING NETWORK")
        cmd = ["nmcli", "networking", "off"]
        try:
            subprocess.run(cmd, check=True)
            self.logger.info("Network killed successfully.")
        except Exception as e:
            self.logger.critical(f"FAILED TO KILL NETWORK: {e}")
            # If nmcli fails, try ip link set down as backup?
            # self.logger.critical("Attempting backup kill method...")
            # subprocess.run(["ip", "link", "set", "wlan0", "down"])
