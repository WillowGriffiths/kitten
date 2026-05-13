build-opensbi: clean-opensbi
    make -C opensbi -j`nproc` PLATFORM=generic LLVM=1
    @cp ./opensbi/build/platform/generic/firmware/fw_dynamic.bin ./opensbi_v1.8.1.bin

clean-opensbi:
    make -C opensbi clean
