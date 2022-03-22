# danmu2ass
[![Continuous integration](https://github.com/gwy15/danmu2ass/actions/workflows/ci.yml/badge.svg)](https://github.com/gwy15/danmu2ass/actions/workflows/ci.yml)
[![Publish Docker image](https://github.com/gwy15/danmu2ass/actions/workflows/docker.yml/badge.svg)](https://github.com/gwy15/danmu2ass/actions/workflows/docker.yml)

将哔哩哔哩的 xml 文件转化为 ass 文件

## 特性
- 比 danmaku2ass 快一百倍的速度（见下方性能对比）
- 更紧密的弹幕填充算法（见下）
- 底部和顶部弹幕和逆向弹幕转成正常弹幕

![填充算法示例](./.github/sample.png)

## 性能对比
xml 解析器默认使用 quick_xml

测试 238M 的文件：
- `quick-xml`：449.5ms 461.3ms 505.1ms 406.23ms
- `xml-rs`：18.0s 18.8s 18.2s 18.5s
- `danmaku2ass`：40.2s 40.8s 40.1s

> danmuku2ass 使用命令行
> 
> `python3 ../danmaku2ass/danmaku2ass.py -f Bilibili -s 1280x720 ./large.xml -o large.ass`

# 安装
- 下载 https://github.com/gwy15/danmu2ass/releases 中的 release
- 或者使用 cargo 安装（如果你有 cargo）：`cargo install danmu2ass`
- 或者使用 docker：`docker run -it --rm -v /tmp:/tmp gwy15/danmu2ass:main /tmp/input.xml`
