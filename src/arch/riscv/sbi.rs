#[allow(dead_code)]
mod consts {
    // SBI Extension IDs (EID)
    pub const EID_BASE: usize = 0x10;
    pub const EID_TIMER: usize = 0x54494D45; // "TIME"
    pub const EID_IPI: usize = 0x735049; // "sPI"
    pub const EID_RFENCE: usize = 0x52464E43; // "RFNC"
    pub const EID_HSM: usize = 0x48534D; // "HSM"
    pub const EID_SRST: usize = 0x53525354; // "SRST"
    pub const EID_PMU: usize = 0x504D55; // "PMU"
    pub const EID_DBCN: usize = 0x4442434E; // "DBCN" - Debug Console
    pub const EID_SUSP: usize = 0x53555350; // "SUSP" - System Suspend
    pub const EID_CPPC: usize = 0x43505043; // "CPPC" - Collaborative Processor Performance Control
    pub const EID_NACL: usize = 0x4E41434C; // "NACL" - Nested Acceleration
    pub const EID_STA: usize = 0x535441; // "STA" - Steal-time Accounting

    // Legacy Extensions (deprecated but still available)
    pub const EID_LEGACY_SET_TIMER: usize = 0x00;
    pub const EID_LEGACY_CONSOLE_PUTCHAR: usize = 0x01;
    pub const EID_LEGACY_CONSOLE_GETCHAR: usize = 0x02;
    pub const EID_LEGACY_CLEAR_IPI: usize = 0x03;
    pub const EID_LEGACY_SEND_IPI: usize = 0x04;
    pub const EID_LEGACY_REMOTE_FENCE_I: usize = 0x05;
    pub const EID_LEGACY_REMOTE_SFENCE_VMA: usize = 0x06;
    pub const EID_LEGACY_REMOTE_SFENCE_VMA_ASID: usize = 0x07;
    pub const EID_LEGACY_SHUTDOWN: usize = 0x08;

    // Base Extension (EID 0x10) Function IDs
    pub const FID_BASE_GET_SPEC_VERSION: usize = 0x0;
    pub const FID_BASE_GET_IMPL_ID: usize = 0x1;
    pub const FID_BASE_GET_IMPL_VERSION: usize = 0x2;
    pub const FID_BASE_PROBE_EXTENSION: usize = 0x3;
    pub const FID_BASE_GET_MVENDORID: usize = 0x4;
    pub const FID_BASE_GET_MARCHID: usize = 0x5;
    pub const FID_BASE_GET_MIMPID: usize = 0x6;

    // Timer Extension (EID 0x54494D45) Function IDs
    pub const FID_TIMER_SET_TIMER: usize = 0x0;

    // IPI Extension (EID 0x735049) Function IDs
    pub const FID_IPI_SEND_IPI: usize = 0x0;

    // RFENCE Extension (EID 0x52464E43) Function IDs
    pub const FID_RFENCE_REMOTE_FENCE_I: usize = 0x0;
    pub const FID_RFENCE_REMOTE_SFENCE_VMA: usize = 0x1;
    pub const FID_RFENCE_REMOTE_SFENCE_VMA_ASID: usize = 0x2;
    pub const FID_RFENCE_REMOTE_HFENCE_GVMA_VMID: usize = 0x3;
    pub const FID_RFENCE_REMOTE_HFENCE_GVMA: usize = 0x4;
    pub const FID_RFENCE_REMOTE_HFENCE_VVMA_ASID: usize = 0x5;
    pub const FID_RFENCE_REMOTE_HFENCE_VVMA: usize = 0x6;

    // Hart State Management Extension (EID 0x48534D) Function IDs
    pub const FID_HSM_HART_START: usize = 0x0;
    pub const FID_HSM_HART_STOP: usize = 0x1;
    pub const FID_HSM_HART_GET_STATUS: usize = 0x2;
    pub const FID_HSM_HART_SUSPEND: usize = 0x3;

    // System Reset Extension (EID 0x53525354) Function IDs
    pub const FID_SRST_SYSTEM_RESET: usize = 0x0;

    // PMU Extension (EID 0x504D55) Function IDs
    pub const FID_PMU_NUM_COUNTERS: usize = 0x0;
    pub const FID_PMU_COUNTER_GET_INFO: usize = 0x1;
    pub const FID_PMU_COUNTER_CONFIG_MATCHING: usize = 0x2;
    pub const FID_PMU_COUNTER_START: usize = 0x3;
    pub const FID_PMU_COUNTER_STOP: usize = 0x4;
    pub const FID_PMU_COUNTER_FW_READ: usize = 0x5;
    pub const FID_PMU_COUNTER_FW_READ_HI: usize = 0x6;
    pub const FID_PMU_SNAPSHOT_SET_SHMEM: usize = 0x7;

    // Debug Console Extension (EID 0x4442434E) Function IDs
    pub const FID_DBCN_CONSOLE_WRITE: usize = 0x0;
    pub const FID_DBCN_CONSOLE_READ: usize = 0x1;
    pub const FID_DBCN_CONSOLE_WRITE_BYTE: usize = 0x2;

    // System Suspend Extension (EID 0x53555350) Function IDs
    pub const FID_SUSP_SYSTEM_SUSPEND: usize = 0x0;

    // CPPC Extension (EID 0x43505043) Function IDs
    pub const FID_CPPC_PROBE: usize = 0x0;
    pub const FID_CPPC_READ: usize = 0x1;
    pub const FID_CPPC_READ_HI: usize = 0x2;
    pub const FID_CPPC_WRITE: usize = 0x3;

    // Nested Acceleration Extension (EID 0x4E41434C) Function IDs
    pub const FID_NACL_PROBE_FEATURE: usize = 0x0;
    pub const FID_NACL_SET_SHMEM: usize = 0x1;
    pub const FID_NACL_SYNC_CSR: usize = 0x2;
    pub const FID_NACL_SYNC_HFENCE: usize = 0x3;
    pub const FID_NACL_SYNC_SRET: usize = 0x4;

    // Steal-time Accounting Extension (EID 0x535441) Function IDs
    pub const FID_STA_SET_SHMEM: usize = 0x0;

    // SBI Error Codes
    pub const SBI_SUCCESS: isize = 0;
    pub const SBI_ERR_FAILED: isize = -1;
    pub const SBI_ERR_NOT_SUPPORTED: isize = -2;
    pub const SBI_ERR_INVALID_PARAM: isize = -3;
    pub const SBI_ERR_DENIED: isize = -4;
    pub const SBI_ERR_INVALID_ADDRESS: isize = -5;
    pub const SBI_ERR_ALREADY_AVAILABLE: isize = -6;
    pub const SBI_ERR_ALREADY_STARTED: isize = -7;
    pub const SBI_ERR_ALREADY_STOPPED: isize = -8;
    pub const SBI_ERR_NO_SHMEM: isize = -9;

    // System Reset Types (for SRST extension)
    pub const RESET_TYPE_SHUTDOWN: usize = 0x0;
    pub const RESET_TYPE_COLD_REBOOT: usize = 0x1;
    pub const RESET_TYPE_WARM_REBOOT: usize = 0x2;

    // System Reset Reasons (for SRST extension)
    pub const RESET_REASON_NO_REASON: usize = 0x0;
    pub const RESET_REASON_SYSTEM_FAILURE: usize = 0x1;
}

#[inline(always)]
fn sbi_call(
    eid: usize,
    fid: usize,
    arg0: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
) -> (usize, usize) {
    let error: usize;
    let value: usize;

    unsafe {
        core::arch::asm!(
            "ecall",
            in("a0") arg0,
            in("a1") arg1,
            in("a2") arg2,
            in("a3") arg3,
            in("a4") arg4,
            in("a6") fid,
            in("a7") eid,
            lateout("a0") error,
            lateout("a1") value,
        );
    }

    (error, value)
}

pub fn print(s: &str) {
    let virtual_diff: usize = 0xffffffff80000000 - 0x80000000;

    let bytes = s.as_bytes();
    let ptr = bytes.as_ptr() as usize;
    let len = bytes.len();

    sbi_call(
        consts::EID_DBCN,
        consts::FID_DBCN_CONSOLE_WRITE,
        len,
        ptr - virtual_diff,
        0,
        0,
        0,
    );
}
