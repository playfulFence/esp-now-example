# esp-now-example :crab:
Example for real HW for communication between two boards (or more) with [esp-now](https://www.espressif.com/en/products/software/esp-now/overview). In terms of this example [RUST-BOARD](https://github.com/esp-rs/esp-rust-board) will send temperature or accelerometer data from it's on-board temperature sensor to any another board (in fact, broadcasts it)

This repo consists of two examples for `sender` and `receiver` chip with two corresponding source files in `/examples` directory of this repo.

This repo will be changing as `esp-wifi` driver isn't in it's final state yet

>### **Important** : In fact every member of connection is sender and receiver (you can see it in code also), but I'll keep name one of them sender in order to make it easier to understand the things :heart:

## Instructions

Don't forget about (un)commenting corresponding dependencies for used chip!

### Temperature

Execution command for sender (`RUST-BOARD` in our case):
```
cargo +nightly espflash --example embassy_esp_now_send_temp --release --target riscv32imc-unknown-none-elf --monitor
```

Execution command for receivers (`RISC`):
```
cargo +nightly espflash --example embassy_esp_now_receive_temp --release --target riscv32imc-unknown-none-elf --monitor
```

Execution command for receivers (`XTENSA`):
```
cargo +esp espflash --example embassy_esp_now_receive_temp --release --target xtensa-<chip_name>-none-elf --monitor
```
---

### Accelerometer

Execution command for sender (`RUST-BOARD` in our case):
```
cargo +nightly espflash --example embassy_esp_now_send_accel --release --target riscv32imc-unknown-none-elf --monitor
```

Execution command for receivers (`RISC`):
```
cargo +nightly espflash --example embassy_esp_now_receive_accel --release --target riscv32imc-unknown-none-elf --monitor
```

Execution command for receivers (`XTENSA`):
```
cargo +esp espflash --example embassy_esp_now_receive_accel --release --target xtensa-<chip_name>-none-elf --monitor
```


In case you don't have corresponding environment, take a look at [espup](https://github.com/esp-rs/espup).


In case if something is unclear for you or you've noticed weird/wrong behaviour feel free to open an issue
