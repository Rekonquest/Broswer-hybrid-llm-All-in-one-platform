import unittest
from unittest.mock import patch, MagicMock
import os
import sys

# Add src to path
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '../src')))

from ghost_symphony.mock_system import BaitIdentity
from ghost_symphony.network import NetworkGhost
from ghost_symphony.sandbox import SandboxManager
from ghost_symphony.gatekeeper import Gatekeeper

class TestGhostSymphony(unittest.TestCase):
    """
    Dry-run tests for the entire Ghost Symphony system.
    This simulates the "Jules" environment verification.
    """

    def test_mock_system_bait_deployment(self):
        bait = BaitIdentity(debug_mode=True)
        # Verify it doesn't crash
        bait.deploy_bait("/tmp/test_bait")

    @patch("subprocess.run")
    def test_network_ghost_commands(self, mock_run):
        net = NetworkGhost()
        net.rotate_mac("wlan0")
        mock_run.assert_called()

        net.harden_ttl()
        mock_run.assert_called()

    def test_sandbox_command_generation(self):
        sandbox = SandboxManager()
        cmd = sandbox.build_bwrap_command("/tmp/bait", ["firefox"])
        self.assertIn("bwrap", cmd)
        self.assertIn("--unshare-all", cmd)
        self.assertIn("LIBGL_ALWAYS_SOFTWARE", cmd)

    @patch("ghost_symphony.gatekeeper.CDLL")
    def test_gatekeeper_mock_init(self, mock_cdll):
        # Simulate libc not found or mock it
        mock_libc = MagicMock()
        mock_cdll.return_value = mock_libc
        mock_libc.fanotify_init.return_value = 123

        gk = Gatekeeper("/tmp")
        # Just check if we can instantiate it and it tries to load libc
        self.assertIsNotNone(gk)

if __name__ == '__main__':
    unittest.main()
