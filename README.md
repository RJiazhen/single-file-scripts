# single-file-scripts

个人编写的各种脚本，所有的脚本遵循只使用一个文件进行编写，只解决一个特定的问题。

脚本优先保证在 windows 环境下正常运行，除了`cmd`脚本，其他脚本不保证能直接运行，请根据说明安装相关运行环境和依赖。

## 脚本说明

### `scheduled-shutdown.bat`

windows 系统中设置定时关机

### `webpToJpg.py`

将 zip 压缩包内所有 webp 图片转换为 jpg 格式图片

运行环境：`Python 3.11`
相关依赖：`send2trash`、`pillow`

### `rime_dict_add.rs`

Rime 词库追加工具。将「文字+制表符+编码」格式的输入追加到词库文件末尾。

运行：`cargo run --bin rime-dict-add` 或 `./target/release/rime-dict-add`
首次运行会提示输入词库路径，配置保存在脚本同目录的 `.rime_dict_config` 中。
