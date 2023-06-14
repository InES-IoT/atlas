# :crab: Atlas - Exploring statically linked ELF files :crab:

Analysis tool for statically linked ELF files for exploring memory usage in regards to language (C, Cpp, Rust), memory regions (ROM, RAM), and memory sections (BSS, text, data, ...).

---

## Examples

### Print a memory usage summary
```
❯ atlas --nm <nm path> --elf <elf path> --rlib <rust lib path> -s
 Rom  | Size [Bytes] | %age
------+--------------+------
 Cpp  | 128.6 kiB    | 49.6
 C    | 125.9 kiB    | 48.5
 Rust | 4.9 kiB      | 1.9
```

### List 5 largest symbols in ROM
```
❯ atlas --nm <nm path> --elf <elf path> --rlib <rust lib path> -c 5
 Language | Name                                       | Size [Bytes] | Symbol Type         | Memory Region
----------+--------------------------------------------+--------------+---------------------+---------------
 C        | cbvprintf                                  | 3332         | TextSection         | Rom
 C        | shell_process                              | 1720         | TextSection         | Rom
 Cpp      | ot::Mle::MleRouter::HandleAdvertisement(ot | 1076         | TextSection         | Rom
          | ::Message const&, ot::Ip6::MessageInfo     |              |                     |
          | const&, ot::Neighbor*)                     |              |                     |
 C        | RT3                                        | 1024         | ReadOnlyDataSection | Rom
 C        | RT2                                        | 1024         | ReadOnlyDataSection | Rom
```


### List 10 largest symbols in RAM
```
❯ atlas --nm <nm path> --elf <elf path> --rlib <rust lib path> -c 10 -r ram
 Language | Name                  | Size [Bytes] | Symbol Type | Memory Region
----------+-----------------------+--------------+-------------+---------------
 Cpp      | ot::gInstanceRaw      | 26608        | BssSection  | Ram
 C        | z_main_stack          | 4128         | BssSection  | Ram
 C        | test_arr              | 4096         | BssSection  | Ram
 C        | kheap__system_heap    | 4096         | BssSection  | Ram
 C        | shell_uart_stack      | 3104         | BssSection  | Ram
 C        | ot_stack_area         | 3104         | BssSection  | Ram
 C        | z_interrupt_stacks    | 2176         | BssSection  | Ram
 C        | sys_work_q_stack      | 2080         | BssSection  | Ram
 C        | nrf_802154_rx_buffers | 2064         | BssSection  | Ram
 C        | net_buf_data_tx_bufs  | 2048         | BssSection  | Ram
```

## Installation

[Install Rust](https://www.rust-lang.org/tools/install), clone the repo, and install the tool using cargo:

```
git clone https://github.zhaw.ch/InESTeamIOT/rust4iot.git
cd rust4iot/tools/atlas
cargo install --path .
```

## Minimum Supported Rust Version (MSRV)
Requires Rust version `1.56.1` and later.
