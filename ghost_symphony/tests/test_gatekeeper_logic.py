import unittest
from unittest.mock import MagicMock, patch
import os
from ghost_symphony.src.ghost_symphony.gatekeeper import Gatekeeper

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
        # The current mock implementation returns True (Simulating YES), but logs a warning.
        # We can verify it logs.
        with self.assertLogs("Gatekeeper", level='WARNING') as cm:
            result = self.gatekeeper._prompt_user("/etc/passwd", 1234)
            self.assertTrue(result) # Still true in simulation
            self.assertTrue(any("SECURITY ALERT" in o for o in cm.output))

    def test_resolve_path(self):
        # We can't easily test os.readlink on /proc/self/fd/X without a real fd.
        # But we can patch it.
        with patch("os.readlink", return_value="/test/path"):
            path = self.gatekeeper._resolve_path(99)
            self.assertEqual(path, "/test/path")

if __name__ == '__main__':
    unittest.main()
