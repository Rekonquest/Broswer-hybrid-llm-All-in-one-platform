import unittest
from unittest.mock import MagicMock, patch, mock_open
import os
from ghost_symphony.src.ghost_symphony.gatekeeper import Gatekeeper, SecurityBreach

class TestGatekeeperLogic(unittest.TestCase):
    def setUp(self):
        self.gatekeeper = Gatekeeper("/tmp")

    @patch("os.readlink")
    def test_prompt_logic_allow_ghost(self, mock_readlink):
        # Simulate access to ~/Ghost/file.txt
        ghost_file = os.path.expanduser("~/Ghost/secret.txt")
        result = self.gatekeeper._prompt_user(ghost_file, 1234)
        self.assertTrue(result)

    @patch("os.readlink")
    def test_prompt_logic_warn_others(self, mock_readlink):
        # Simulate access to /etc/passwd
        with self.assertLogs("Gatekeeper", level='WARNING') as cm:
            result = self.gatekeeper._prompt_user("/etc/passwd", 1234)
            self.assertTrue(result)
            self.assertTrue(any("SECURITY ALERT" in o for o in cm.output))

    def test_resolve_path(self):
        with patch("os.readlink", return_value="/test/path"):
            path = self.gatekeeper._resolve_path(99)
            self.assertEqual(path, "/test/path")

    def test_pid_tree_tracking(self):
        self.gatekeeper.set_target_pid(100)
        self.assertTrue(self.gatekeeper._is_monitored_pid(100))
        self.assertFalse(self.gatekeeper._is_monitored_pid(999))

        # Mock /proc/{pid}/stat reading for a child process 101 -> parent 100
        # Format: 101 (comm) S 100 ...
        stat_content = "101 (bash) S 100 1 2 3"

        with patch("builtins.open", mock_open(read_data=stat_content)):
            # Check child
            self.assertTrue(self.gatekeeper._is_monitored_pid(101))
            # Verify cache
            self.assertIn(101, self.gatekeeper.monitored_pids)

    @patch("ghost_symphony.src.ghost_symphony.gatekeeper.os.read")
    def test_handle_event_security_breach(self, mock_read):
        # Simulate read error
        mock_read.side_effect = OSError("Kernel error")

        with self.assertRaises(SecurityBreach):
            self.gatekeeper.handle_event()

if __name__ == '__main__':
    unittest.main()
