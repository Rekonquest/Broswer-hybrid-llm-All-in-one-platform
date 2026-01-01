import os
import logging

class BaitIdentity:
    """
    Manages the creation of fake hardware identity files (cpuinfo, meminfo)
    to mask the true system specs from the sandboxed process.
    """

    # Mimicking Intel i5-1135G7
    CPU_INFO_TEMPLATE = """processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 140
model name	: 11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz
stepping	: 1
microcode	: 0xb4
cpu MHz		: 2400.000
cache size	: 8192 KB
physical id	: 0
siblings	: 8
core id		: 0
cpu cores	: 4
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush dts acpi mmx fxsr sse sse2 ss ht tm pbe syscall nx pdpe1gb rdtscp lm constant_tsc art arch_perfmon pebs bts rep_good nopl xtopology nonstop_tsc cpuid aperfmperf pni pclmulqdq dtes64 monitor ds_cpl vmx est tm2 ssse3 sdbg fma cx16 xtpr pdcm pcid sse4_1 sse4_2 x2apic movbe popcnt tsc_deadline_timer aes xsave avx f16c rdrand lahf_lm abm 3dnowprefetch cpuid_fault epb invpcid_single ssbd ibrs ibpb stibp ibrs_enhanced tpr_shadow vnmi flexpriority ept vpid ept_ad fsgsbase tsc_adjust bmi1 avx2 smep bmi2 erms invpcid mpx avx512f avx512dq rdseed adx smap clflushopt clwb intel_pt avx512cd avx512bw avx512vl xsaveopt xsavec xgetbv1 xsaves dtherm ida arat pln pts hwp hwp_notify hwp_act_window hwp_epp md_clear flush_l1d arch_capabilities
"""
    # Mimicking 8GB RAM
    MEM_INFO_TEMPLATE = """MemTotal:        8165420 kB
MemFree:         3245120 kB
MemAvailable:    5124000 kB
Buffers:          124500 kB
Cached:          2104500 kB
SwapCached:            0 kB
Active:          2456000 kB
Inactive:        1850000 kB
"""

    def __init__(self, debug_mode: bool = False):
        self.debug_mode = debug_mode
        self.logger = logging.getLogger("MockSystem")
        if self.debug_mode:
            logging.basicConfig(level=logging.DEBUG)

    def deploy_bait(self, target_dir: str):
        """
        Deploys the fake cpuinfo and meminfo files to the target directory.
        """
        self.logger.info(f"Preparing to deploy bait identity to {target_dir}...")

        cpu_path = os.path.join(target_dir, "cpuinfo")
        mem_path = os.path.join(target_dir, "meminfo")

        # In debug mode, we just print what we would have done
        if self.debug_mode:
            self.logger.debug(f"--- [SELF_DEBUG] Would write to {cpu_path} ---")
            self.logger.debug(self.CPU_INFO_TEMPLATE[:200] + "... (truncated)")
            self.logger.debug(f"--- [SELF_DEBUG] Would write to {mem_path} ---")
            self.logger.debug(self.MEM_INFO_TEMPLATE)
            return

        try:
            # Ensure directory exists
            os.makedirs(target_dir, exist_ok=True)

            # Generate full CPU info (replicating for 8 siblings/threads as implied by template)
            # The template provided is for processor : 0. We should replicate it for 0-7?
            # The prompt said "... (repeat for processors 1-7)".
            full_cpu_info = ""
            for i in range(8):
                # Replace "processor	: 0" with "processor	: i"
                # The template starts with "processor	: 0".
                # We will just replace the first line.
                entry = self.CPU_INFO_TEMPLATE.replace("processor	: 0", f"processor	: {i}", 1)
                full_cpu_info += entry + "\n"

            with open(cpu_path, "w") as f:
                f.write(full_cpu_info)
            self.logger.info(f"Deployed fake cpuinfo to {cpu_path}")

            with open(mem_path, "w") as f:
                f.write(self.MEM_INFO_TEMPLATE)
            self.logger.info(f"Deployed fake meminfo to {mem_path}")

        except OSError as e:
            self.logger.error(f"Failed to deploy bait: {e}")
            raise
