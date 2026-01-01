import unittest
from unittest.mock import patch, MagicMock
import os
from ghost_symphony.src.ghost_symphony.sandbox import SandboxManager

class TestSandboxManager(unittest.TestCase):
    def setUp(self):
        self.sandbox = SandboxManager()

    @patch("subprocess.run")
    def test_setup_audio_sink(self, mock_run):
        self.sandbox.setup_audio_sink()
        mock_run.assert_called_with(
            ['pactl', 'load-module', 'module-null-sink',
             'sink_name=Ghost_Mic',
             'media.class=Audio/Source/Virtual',
             'node.description="Integrated Microphone"'],
            check=True, capture_output=True
        )

    def test_build_bwrap_command(self):
        target_dir = "/tmp/fake_sys"
        browser_cmd = ["firefox"]
        cmd = self.sandbox.build_bwrap_command(target_dir, browser_cmd)

        self.assertIn("bwrap", cmd)
        self.assertIn("--ro-bind", cmd)
        self.assertIn("/proc/cpuinfo", cmd)
        self.assertIn("/proc/meminfo", cmd)
        self.assertIn("--setenv", cmd)
        self.assertIn("LIBGL_ALWAYS_SOFTWARE", cmd)
        self.assertIn("ghost-station", cmd)
        self.assertIn("firefox", cmd)

        # Check specific bindings
        self.assertIn(os.path.join(target_dir, "cpuinfo"), cmd)

if __name__ == '__main__':
    unittest.main()
