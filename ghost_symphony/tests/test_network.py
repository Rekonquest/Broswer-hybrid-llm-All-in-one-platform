import unittest
from unittest.mock import patch, MagicMock
from ghost_symphony.src.ghost_symphony.network import NetworkGhost

class TestNetworkGhost(unittest.TestCase):
    def setUp(self):
        self.ghost = NetworkGhost()

    @patch("subprocess.run")
    def test_rotate_mac(self, mock_run):
        self.ghost.rotate_mac("wlan0")
        mock_run.assert_called_with(
            ["nmcli", "device", "modify", "wlan0", "802-11-wireless.cloned-mac-address", "random"],
            check=True, capture_output=True, text=True
        )

    @patch("subprocess.run")
    def test_harden_ttl(self, mock_run):
        self.ghost.harden_ttl()
        mock_run.assert_called_with(
            ["sysctl", "-w", "net.ipv4.ip_default_ttl=128"],
            check=True, capture_output=True, text=True
        )

    @patch("subprocess.run")
    def test_panic_shutdown(self, mock_run):
        self.ghost.panic_shutdown()
        mock_run.assert_called_with(
            ["nmcli", "networking", "off"],
            check=True
        )

    @patch("subprocess.run")
    def test_command_failure(self, mock_run):
        # Simulate a command failure
        mock_run.side_effect = subprocess.CalledProcessError(1, "cmd", stderr="error")
        # Should catch exception and log error, not crash
        with self.assertLogs("NetworkGhost", level='ERROR') as cm:
            self.ghost.harden_ttl()
        self.assertIn("Failed to set default TTL to 128", cm.output[0])

import subprocess
if __name__ == '__main__':
    unittest.main()
