# danmu2ass
[![Continuous integration](https://github.com/gwy15/danmu2ass/actions/workflows/ci.yml/badge.svg)](https://github.com/gwy15/danmu2ass/actions/workflows/ci.yml)
[![Publish Docker image](https://github.com/gwy15/danmu2ass/actions/workflows/docker.yml/badge.svg)](https://github.com/gwy15/danmu2ass/actions/workflows/docker.yml)

将哔哩哔哩的 xml 文件转化为 ass 文件

## 特性
- 比 danmaku2ass 快一百倍的速度（见下方性能对比）
- 更紧密的弹幕填充算法（见下）
- 底部和顶部弹幕和逆向弹幕转成正常弹幕，减少遮挡
- 弹幕透明度、字体、字号、高度、间距全部可调
- 支持过滤黑名单关键词
- 支持文件夹模式，递归查找所有 xml 文件并多线程处理
- 自动判断是否已经转换过，跳过已转换的文件，方便自动化处理
- 编译为二进制，支持 docker 部署，不需要 python 环境

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
    - 对于 Windows 用户，默认 zip 内会有一个 配置文件.toml，更改其中内容即可更改配置。
    - 该配置文件存在时不会解析命令行输入
- 或者使用 cargo 安装（如果你有 cargo）：`cargo install danmu2ass`
- 或者使用 docker：`docker run -it --rm -v /tmp:/tmp gwy15/danmu2ass:main /tmp/input.xml`

# 使用
```plaintext
danmu2ass 0.1.7
gwy15
将 XML 弹幕转换为 ASS 文件

USAGE:
    danmu2ass [OPTIONS] [XML_FILE_OR_PATH]

ARGS:
    <XML_FILE_OR_PATH>    需要转换的 XML 文件或文件夹，如果是文件夹会递归将其下所有 XML
                          都进行转换 [default: .]

OPTIONS:
    -a, --alpha <ALPHA>
            弹幕不透明度 [default: 0.7]

        --bold
            加粗

    -d, --duration <DURATION>
            弹幕在屏幕上的持续时间，单位为秒，可以有小数 [default: 15]

        --denylist <DENYLIST>
            黑名单，需要过滤的关键词列表文件，每行一个关键词

    -f, --font <FONT>
            弹幕使用字体 [default: 黑体]

        --font-size <FONT_SIZE>
            弹幕字体大小 [default: 25]

        --force
            默认会跳过 ass 比 xml 修改时间更晚的文件，此参数会强制转换

    -h, --height <HEIGHT>
            屏幕高度 [default: 720]

        --help
            Print help information

    -l, --lane-size <LANE_SIZE>
            弹幕所占据的高度，即“行高度/行间距” [default: 32]

    -o, --output <ASS_FILE>
            输出的 ASS 文件，默认为输入文件名将 .xml 替换为 .ass，如果输入是文件夹则忽略

        --outline <OUTLINE>
            描边宽度 [default: 0.8]

    -p, --float-percentage <FLOAT_PERCENTAGE>
            屏幕上滚动弹幕最多高度百分比 [default: 0.5]

        --pause
            在处理完后暂停等待输入

        --time-offset <TIME_OFFSET>
            时间轴偏移，>0 会让弹幕延后，<0 会让弹幕提前，单位为秒 [default: 0.0]

    -V, --version
            Print version information

    -w, --width <WIDTH>
            屏幕宽度 [default: 1280]

        --width-ratio <WIDTH_RATIO>
            为避免重叠需要调大这个数值，即“水平间距” [default: 1.2]
```
