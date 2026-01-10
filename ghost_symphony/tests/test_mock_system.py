import unittest
import tempfile
import shutil
import os
from ghost_symphony.src.ghost_symphony.mock_system import BaitIdentity

class TestMockSystem(unittest.TestCase):
    def setUp(self):
        self.test_dir = tempfile.mkdtemp()
        self.bait = BaitIdentity(debug_mode=False)

    def tearDown(self):
        shutil.rmtree(self.test_dir)

    def test_deploy_bait_files_exist(self):
        self.bait.deploy_bait(self.test_dir)
        self.assertTrue(os.path.exists(os.path.join(self.test_dir, "cpuinfo")))
        self.assertTrue(os.path.exists(os.path.join(self.test_dir, "meminfo")))

    def test_cpuinfo_content(self):
        self.bait.deploy_bait(self.test_dir)
        with open(os.path.join(self.test_dir, "cpuinfo"), "r") as f:
            content = f.read()

        # Check if it generated 8 processors (0 to 7)
        self.assertIn("processor\t: 0", content)
        self.assertIn("processor\t: 7", content)
        self.assertIn("GenuineIntel", content)
        self.assertIn("i5-1135G7", content)

    def test_meminfo_content(self):
        self.bait.deploy_bait(self.test_dir)
        with open(os.path.join(self.test_dir, "meminfo"), "r") as f:
            content = f.read()

        self.assertIn("MemTotal:        8165420 kB", content)

    def test_debug_mode(self):
        debug_bait = BaitIdentity(debug_mode=True)
        # Should not write files
        debug_bait.deploy_bait(self.test_dir)
        self.assertFalse(os.path.exists(os.path.join(self.test_dir, "cpuinfo")))

if __name__ == '__main__':
    unittest.main()
