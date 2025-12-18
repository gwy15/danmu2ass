use super::danmu::{Danmu, DanmuType};
use anyhow::{bail, Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Seek},
    path::Path,
};

#[cfg(feature = "quick_xml")]
use quick_xml::Reader;
#[cfg(feature = "xml_rs")]
use xml::reader::EventReader as Reader;

pub struct Parser<R: BufRead> {
    count: usize,
    reader: Reader<R>,
    #[cfg(feature = "quick_xml")]
    buf: Vec<u8>,
}

impl<R: BufRead> Parser<R> {
    pub fn new(reader: R) -> Self {
        #[cfg(feature = "xml_rs")]
        let reader = Reader::new(reader);
        #[cfg(feature = "quick_xml")]
        let reader = Reader::from_reader(reader);

        Self {
            count: 0,
            reader,

            #[cfg(feature = "quick_xml")]
            buf: Vec::new(),
        }
    }
}

impl Parser<BufReader<File>> {
    pub fn from_path(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        // 对于 HDD、docker 之类的场景，磁盘 IO 是非常大的瓶颈。使用大缓存
        let mut reader = BufReader::with_capacity(10 << 20, file);
        let mut bom_buf = [0u8; 3];
        reader.read_exact(&mut bom_buf)?;
        if bom_buf != [0xEF, 0xBB, 0xBF] {
            reader.seek(std::io::SeekFrom::Start(0))?;
        }

        #[cfg(feature = "xml_rs")]
        let reader = Reader::new(reader);
        #[cfg(feature = "quick_xml")]
        let reader = Reader::from_reader(reader);

        Ok(Self {
            count: 0,
            reader,
            #[cfg(feature = "quick_xml")]
            buf: Vec::new(),
        })
    }
}

impl<R: BufRead> Iterator for Parser<R> {
    type Item = Result<Danmu>;

    #[cfg(feature = "xml_rs")]
    fn next(&mut self) -> Option<Result<Danmu>> {
        let mut danmu = Danmu::default();
        loop {
            let event = match self.reader.next().context("XML 文件解析错误") {
                Ok(e) => e,
                Err(e) => return Some(Err(e)),
            };
            match event {
                xml::reader::XmlEvent::EndDocument => {
                    return None;
                }
                xml::reader::XmlEvent::StartElement {
                    name, attributes, ..
                } if name.local_name == "d" => {
                    let p_attr = match attributes
                        .into_iter()
                        .find(|attr| attr.name.local_name == "p")
                    {
                        Some(p_attr) => p_attr,
                        None => {
                            return Some(Err(anyhow::anyhow!(
                                "弹幕 <d> 中没找到 p 属性，xml 文件可能有错误"
                            )))
                        }
                    };

                    match Danmu::from_xml_p_attr(&p_attr.value).context("p 属性解析错误") {
                        Ok(parsed) => {
                            danmu = parsed;
                        }
                        Err(e) => return Some(Err(e)),
                    };
                }
                xml::reader::XmlEvent::EndElement { name } if name.local_name == "d" => {
                    self.count += 1;
                    return Some(Ok(danmu));
                }
                xml::reader::XmlEvent::Characters(s) => {
                    #[cfg(debug_assertions)]
                    {
                        danmu.content = format!("{}-{}", self.count, s);
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        danmu.content = s;
                    }
                }
                xml::reader::XmlEvent::StartDocument { .. }
                | xml::reader::XmlEvent::Comment(_)
                | xml::reader::XmlEvent::CData(_)
                | xml::reader::XmlEvent::ProcessingInstruction { .. }
                | xml::reader::XmlEvent::Whitespace(_)
                | xml::reader::XmlEvent::StartElement { .. }
                | xml::reader::XmlEvent::EndElement { .. } => {
                    continue;
                }
            }
        }
    }

    #[cfg(feature = "quick_xml")]
    fn next(&mut self) -> Option<Result<Danmu>> {
        use quick_xml::events::Event;

        /// 一个简单的状态机
        enum Status {
            // on <d> -> AttrWaitForContent
            Start,
            // on text -> WaitForEnd
            AttrWaitForContent(Danmu),
            // on </d> -> return
            WaitForEnd(Danmu),
        }

        let mut status = Status::Start;
        loop {
            let event = self
                .reader
                .read_event_into(&mut self.buf)
                .context("XML 文件解析错误");
            let event = match event {
                Ok(e) => e,
                Err(e) => return Some(Err(e)),
            };

            match event {
                Event::Eof => {
                    return None;
                }
                Event::Start(start) if start.local_name().as_ref() == b"d" => {
                    let p_attr = start
                        .attributes()
                        .filter_map(|r| r.ok())
                        .find(|attr| attr.key.as_ref() == b"p");
                    let Some(p_attr) = p_attr else {
                        return Some(Err(anyhow::anyhow!(
                            "弹幕 <d> 中没找到 p 属性，xml 文件可能有错误"
                        )));
                    };
                    let p_attr_s = match std::str::from_utf8(p_attr.value.as_ref())
                        .context("非法 UTF-8 字符")
                    {
                        Ok(p_attr_s) => p_attr_s,
                        Err(e) => return Some(Err(e)),
                    };

                    match Danmu::from_xml_p_attr(p_attr_s).context("p 属性解析错误") {
                        Ok(Some(parsed)) => {
                            status = Status::AttrWaitForContent(parsed);
                        }
                        Ok(None) => {
                            status = Status::Start;
                        }
                        Err(e) => return Some(Err(e)),
                    };
                }
                Event::End(end) if end.local_name().as_ref() == b"d" => match status {
                    Status::WaitForEnd(danmu) => {
                        self.count += 1;
                        return Some(Ok(danmu));
                    }
                    _ => continue,
                },
                Event::Text(text) => {
                    let s = match std::str::from_utf8(&text).context("非法 UTF-8 字符") {
                        #[cfg(debug_assertions)]
                        Ok(s) => format!("{}-{}", self.count, s),
                        #[cfg(not(debug_assertions))]
                        Ok(s) => s.to_string(),
                        Err(e) => return Some(Err(e)),
                    };
                    match status {
                        Status::AttrWaitForContent(mut danmu) => {
                            danmu.content = s;
                            status = Status::WaitForEnd(danmu);
                        }
                        _ => continue,
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }
}

impl Danmu {
    /// 从哔哩哔哩的弹幕格式解析
    ///
    /// <d p="p" user="user"> content </d>
    /// 其中，p = 0.581,1,25,14893055,1647777083220,0,398452452,0
    /// 分别为：
    /// 1. 时间（秒），
    /// 2. 弹幕类型，（1 为普通弹幕，4 为底部弹幕，5 对应顶部，6 对应反向弹幕）
    /// 3. 字体大小（默认25）
    /// 4. 弹幕颜色（如14893055）
    /// 5. 弹幕毫秒级时间戳（如 1647777083220）
    /// 6. 0
    /// 7. 用户 UID（如 398452452）
    /// 8. 0
    pub fn from_xml_p_attr(p_attr: &str) -> Result<Option<Self>> {
        let mut iter = p_attr.split(',');
        let timeline_s = iter
            .next()
            .context("p 属性中没有时间")?
            .parse()
            .context("时间解析错误")?;
        let r#type = iter
            .next()
            .context("p 属性中没有弹幕类型")?
            .parse()
            .context("弹幕类型解析错误")?;
        let Ok(r#type) = DanmuType::from_xml_num(r#type) else {
            return Ok(None);
        };
        let fontsize: u32 = iter
            .next()
            .context("p 属性中没有字体大小")?
            .parse()
            .context("字体大小解析错误")?;

        let rgb: u32 = iter
            .next()
            .context("p 属性中没有颜色")?
            .parse()
            .context("颜色解析错误")?;
        // rgb 是个数字，一般情况下为 0xRRGGBB，但是偶尔也有 RRRGGGBBB(dec)
        let (r, g, b) = if (rgb >> 24) == 0 {
            ((rgb >> 16) & 0xff, (rgb >> 8) & 0xff, rgb & 0xff)
        } else if rgb <= 255255255 {
            // 见 https://github.com/gwy15/danmu2ass/issues/17，可能有 RRRGGGBBB 的情况
            const K: u32 = 1000;
            (
                ((rgb / K / K) % K) & 0xff,
                ((rgb / K) % K) & 0xff,
                (rgb % K) & 0xff,
            )
        } else {
            bail!("颜色解析错误：颜色为 {:x}", rgb);
        };

        Ok(Some(Self {
            timeline_s,
            content: String::new(),
            r#type,
            fontsize,
            rgb: (r as u8, g as u8, b as u8),
        }))
    }
}
impl DanmuType {
    pub fn from_xml_num(num: u32) -> Result<Self> {
        Ok(match num {
            1 => DanmuType::Float,
            4 => DanmuType::Bottom,
            5 => DanmuType::Top,
            6 => DanmuType::Reverse,
            _ => bail!("未知的弹幕类型：{}", num),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static DATA: &str = r##"
        <?xml version="1.0" encoding="utf-8"?>
        <?xml-stylesheet type="text/xsl" href="#s"?>
        <i>
        <!--
        B站录播姬 1.3.11
        本文件的弹幕信息兼容B站主站视频弹幕XML格式
        本XML自带样式可以在浏览器里打开（推荐使用Chrome）

        sc 为SuperChat
        gift为礼物
        guard为上船

        attribute "raw" 为原始数据
        -->
        <chatserver>chat.bilibili.com</chatserver>
        <chatid>0</chatid>
        <mission>0</mission>
        <maxlimit>1000</maxlimit>
        <state>0</state>
        <real_name>0</real_name>
        <source>0</source>
        <BililiveRecorder version="1.3.11" />
        <BililiveRecorderRecordInfo roomid="22637261" shortid="0" name="嘉然今天吃什么" title="【B限】第一届枝江gamer争霸赛！" areanameparent="虚拟主播" areanamechild="虚拟主播" start_time="2022-03-20T19:51:23.6348295+08:00" />
        <BililiveRecorderXmlStyle><z:stylesheet version="1.0" id="s" xml:id="s" xmlns:z="http://www.w3.org/1999/XSL/Transform"><z:output method="html"/><z:template match="/"><html><meta name="viewport" content="width=device-width"/><title>B站录播姬弹幕文件 - <z:value-of select="/i/BililiveRecorderRecordInfo/@name"/></title><style>body{margin:0}h1,h2,p,table{margin-left:5px}table{border-spacing:0}td,th{border:1px solid grey;padding:1px}th{position:sticky;top:0;background:#4098de}tr:hover{background:#d9f4ff}div{overflow:auto;max-height:80vh;max-width:100vw;width:fit-content}</style><h1>B站录播姬弹幕XML文件</h1><p>本文件的弹幕信息兼容B站主站视频弹幕XML格式，可以使用现有的转换工具把文件中的弹幕转为ass字幕文件</p><table><tr><td>录播姬版本</td><td><z:value-of select="/i/BililiveRecorder/@version"/></td></tr><tr><td>房间号</td><td><z:value-of select="/i/BililiveRecorderRecordInfo/@roomid"/></td></tr><tr><td>主播名</td><td><z:value-of select="/i/BililiveRecorderRecordInfo/@name"/></td></tr><tr><td>录制开始时间</td><td><z:value-of select="/i/BililiveRecorderRecordInfo/@start_time"/></td></tr><tr><td><a href="#d">弹幕</a></td><td>共 <z:value-of select="count(/i/d)"/> 条记录</td></tr><tr><td><a href="#guard">上船</a></td><td>共 <z:value-of select="count(/i/guard)"/> 条记录</td></tr><tr><td><a href="#sc">SC</a></td><td>共 <z:value-of select="count(/i/sc)"/> 条记录</td></tr><tr><td><a href="#gift">礼物</a></td><td>共 <z:value-of select="count(/i/gift)"/> 条记录</td></tr></table><h2 id="d">弹幕</h2><div><table><tr><th>用户名</th><th>弹幕</th><th>参数</th></tr><z:for-each select="/i/d"><tr><td><z:value-of select="@user"/></td><td><z:value-of select="."/></td><td><z:value-of select="@p"/></td></tr></z:for-each></table></div><h2 id="guard">舰长购买</h2><div><table><tr><th>用户名</th><th>舰长等级</th><th>购买数量</th><th>出现时间</th></tr><z:for-each select="/i/guard"><tr><td><z:value-of select="@user"/></td><td><z:value-of select="@level"/></td><td><z:value-of select="@count"/></td><td><z:value-of select="@ts"/></td></tr></z:for-each></table></div><h2 id="sc">SuperChat 醒目留言</h2><div><table><tr><th>用户名</th><th>内容</th><th>显示时长</th><th>价格</th><th>出现时间</th></tr><z:for-each select="/i/sc"><tr><td><z:value-of select="@user"/></td><td><z:value-of select="."/></td><td><z:value-of select="@time"/></td><td><z:value-of select="@price"/></td><td><z:value-of select="@ts"/></td></tr></z:for-each></table></div><h2 id="gift">礼物</h2><div><table><tr><th>用户名</th><th>礼物名</th><th>礼物数量</th><th>出现时间</th></tr><z:for-each select="/i/gift"><tr><td><z:value-of select="@user"/></td><td><z:value-of select="@giftname"/></td><td><z:value-of select="@giftcount"/></td><td><z:value-of select="@ts"/></td></tr></z:for-each></table></div></html></z:template></z:stylesheet></BililiveRecorderXmlStyle>
        <gift ts="0.576" user="粉色羽毛球_Official" giftname="小心心" giftcount="1" raw="{&quot;action&quot;:&quot;投喂&quot;,&quot;batch_combo_id&quot;:&quot;batch:gift:combo_id:197750709:672328094:30607:1647777082.1182&quot;,&quot;batch_combo_send&quot;:null,&quot;beatId&quot;:&quot;&quot;,&quot;biz_source&quot;:&quot;live&quot;,&quot;blind_gift&quot;:null,&quot;broadcast_id&quot;:0,&quot;coin_type&quot;:&quot;silver&quot;,&quot;combo_resources_id&quot;:1,&quot;combo_send&quot;:null,&quot;combo_stay_time&quot;:3,&quot;combo_total_coin&quot;:1,&quot;crit_prob&quot;:0,&quot;demarcation&quot;:1,&quot;discount_price&quot;:0,&quot;dmscore&quot;:28,&quot;draw&quot;:0,&quot;effect&quot;:0,&quot;effect_block&quot;:1,&quot;face&quot;:&quot;http://i0.hdslb.com/bfs/face/9bdc8d6e008efc52721b3056441f6edb27d575fb.jpg&quot;,&quot;float_sc_resource_id&quot;:0,&quot;giftId&quot;:30607,&quot;giftName&quot;:&quot;小心心&quot;,&quot;giftType&quot;:5,&quot;gold&quot;:0,&quot;guard_level&quot;:0,&quot;is_first&quot;:false,&quot;is_special_batch&quot;:0,&quot;magnification&quot;:1,&quot;medal_info&quot;:{&quot;anchor_roomid&quot;:0,&quot;anchor_uname&quot;:&quot;&quot;,&quot;guard_level&quot;:0,&quot;icon_id&quot;:0,&quot;is_lighted&quot;:1,&quot;medal_color&quot;:9272486,&quot;medal_color_border&quot;:9272486,&quot;medal_color_end&quot;:9272486,&quot;medal_color_start&quot;:9272486,&quot;medal_level&quot;:10,&quot;medal_name&quot;:&quot;嘉心糖&quot;,&quot;special&quot;:&quot;&quot;,&quot;target_id&quot;:672328094},&quot;name_color&quot;:&quot;&quot;,&quot;num&quot;:1,&quot;original_gift_name&quot;:&quot;&quot;,&quot;price&quot;:0,&quot;rcost&quot;:200134843,&quot;remain&quot;:10,&quot;rnd&quot;:&quot;1647777083120900002&quot;,&quot;send_master&quot;:null,&quot;silver&quot;:0,&quot;super&quot;:0,&quot;super_batch_gift_num&quot;:5,&quot;super_gift_num&quot;:5,&quot;svga_block&quot;:0,&quot;tag_image&quot;:&quot;&quot;,&quot;tid&quot;:&quot;1647777083120900002&quot;,&quot;timestamp&quot;:1647777083,&quot;top_list&quot;:null,&quot;total_coin&quot;:0,&quot;uid&quot;:197750709,&quot;uname&quot;:&quot;粉色羽毛球_Official&quot;}" />
        <d p="0.581,1,25,14893055,1647777083220,0,398452452,0" user="小马368100" raw="[[0,1,25,14893055,1647777083220,1647776219,0,&quot;1537d7c7&quot;,0,0,5,&quot;#1453BAFF,#4C2263A2,#3353BAFF&quot;,0,&quot;{}&quot;,&quot;{}&quot;,{&quot;mode&quot;:0,&quot;show_player_type&quot;:0,&quot;extra&quot;:&quot;{\&quot;send_from_me\&quot;:false,\&quot;mode\&quot;:0,\&quot;color\&quot;:14893055,\&quot;dm_type\&quot;:0,\&quot;font_size\&quot;:25,\&quot;player_mode\&quot;:1,\&quot;show_player_type\&quot;:0,\&quot;content\&quot;:\&quot;快快快\&quot;,\&quot;user_hash\&quot;:\&quot;355981255\&quot;,\&quot;emoticon_unique\&quot;:\&quot;\&quot;,\&quot;bulge_display\&quot;:0,\&quot;direction\&quot;:0,\&quot;pk_direction\&quot;:0,\&quot;quartet_direction\&quot;:0,\&quot;yeah_space_type\&quot;:\&quot;\&quot;,\&quot;yeah_space_url\&quot;:\&quot;\&quot;,\&quot;jump_to_url\&quot;:\&quot;\&quot;,\&quot;space_type\&quot;:\&quot;\&quot;,\&quot;space_url\&quot;:\&quot;\&quot;}&quot;}],&quot;快快快&quot;,[398452452,&quot;小马368100&quot;,0,0,0,10000,1,&quot;#00D1F1&quot;],[22,&quot;嘉心糖&quot;,&quot;嘉然今天吃什么&quot;,22637261,1725515,&quot;&quot;,0,6809855,1725515,5414290,3,1,672328094],[5,0,9868950,&quot;&gt;50000&quot;,0],[&quot;&quot;,&quot;&quot;],0,3,null,{&quot;ts&quot;:1647777083,&quot;ct&quot;:&quot;7581F4E3&quot;},0,0,null,null,0,105]">快快快</d>
        <d p="0.582,1,25,14893055,1647777083280,0,24755246,0" user="園田" raw="[[0,1,25,14893055,1647777083280,1647776175,0,&quot;1ad255c8&quot;,0,0,5,&quot;#1453BAFF,#4C2263A2,#3353BAFF&quot;,0,&quot;{}&quot;,&quot;{}&quot;,{&quot;mode&quot;:0,&quot;show_player_type&quot;:0,&quot;extra&quot;:&quot;{\&quot;send_from_me\&quot;:false,\&quot;mode\&quot;:0,\&quot;color\&quot;:14893055,\&quot;dm_type\&quot;:0,\&quot;font_size\&quot;:25,\&quot;player_mode\&quot;:1,\&quot;show_player_type\&quot;:0,\&quot;content\&quot;:\&quot;快快快快快快\&quot;,\&quot;user_hash\&quot;:\&quot;449992136\&quot;,\&quot;emoticon_unique\&quot;:\&quot;\&quot;,\&quot;bulge_display\&quot;:0,\&quot;direction\&quot;:0,\&quot;pk_direction\&quot;:0,\&quot;quartet_direction\&quot;:0,\&quot;yeah_space_type\&quot;:\&quot;\&quot;,\&quot;yeah_space_url\&quot;:\&quot;\&quot;,\&quot;jump_to_url\&quot;:\&quot;\&quot;,\&quot;space_type\&quot;:\&quot;\&quot;,\&quot;space_url\&quot;:\&quot;\&quot;}&quot;}],&quot;快快快快快快&quot;,[24755246,&quot;園田&quot;,0,0,0,10000,1,&quot;#00D1F1&quot;],[22,&quot;贝极星&quot;,&quot;贝拉kira&quot;,22632424,1725515,&quot;&quot;,0,6809855,1725515,5414290,3,1,672353429],[27,0,5805790,&quot;&gt;50000&quot;,0],[&quot;&quot;,&quot;&quot;],0,3,null,{&quot;ts&quot;:1647777083,&quot;ct&quot;:&quot;BC30A1B8&quot;},0,0,null,null,0,105]">快快快快快快</d>
        <gift ts="0.582" user="粉色羽毛球_Official" giftname="小心心" giftcount="1" raw="{&quot;action&quot;:&quot;投喂&quot;,&quot;batch_combo_id&quot;:&quot;batch:gift:combo_id:197750709:672328094:30607:1647777082.1182&quot;,&quot;batch_combo_send&quot;:null,&quot;beatId&quot;:&quot;&quot;,&quot;biz_source&quot;:&quot;live&quot;,&quot;blind_gift&quot;:null,&quot;broadcast_id&quot;:0,&quot;coin_type&quot;:&quot;silver&quot;,&quot;combo_resources_id&quot;:1,&quot;combo_send&quot;:null,&quot;combo_stay_time&quot;:3,&quot;combo_total_coin&quot;:1,&quot;crit_prob&quot;:0,&quot;demarcation&quot;:1,&quot;discount_price&quot;:0,&quot;dmscore&quot;:56,&quot;draw&quot;:0,&quot;effect&quot;:0,&quot;effect_block&quot;:1,&quot;face&quot;:&quot;http://i0.hdslb.com/bfs/face/9bdc8d6e008efc52721b3056441f6edb27d575fb.jpg&quot;,&quot;float_sc_resource_id&quot;:0,&quot;giftId&quot;:30607,&quot;giftName&quot;:&quot;小心心&quot;,&quot;giftType&quot;:5,&quot;gold&quot;:0,&quot;guard_level&quot;:0,&quot;is_first&quot;:false,&quot;is_special_batch&quot;:0,&quot;magnification&quot;:1,&quot;medal_info&quot;:{&quot;anchor_roomid&quot;:0,&quot;anchor_uname&quot;:&quot;&quot;,&quot;guard_level&quot;:0,&quot;icon_id&quot;:0,&quot;is_lighted&quot;:1,&quot;medal_color&quot;:9272486,&quot;medal_color_border&quot;:9272486,&quot;medal_color_end&quot;:9272486,&quot;medal_color_start&quot;:9272486,&quot;medal_level&quot;:10,&quot;medal_name&quot;:&quot;嘉心糖&quot;,&quot;special&quot;:&quot;&quot;,&quot;target_id&quot;:672328094},&quot;name_color&quot;:&quot;&quot;,&quot;num&quot;:1,&quot;original_gift_name&quot;:&quot;&quot;,&quot;price&quot;:0,&quot;rcost&quot;:200134843,&quot;remain&quot;:9,&quot;rnd&quot;:&quot;1647777083120900004&quot;,&quot;send_master&quot;:null,&quot;silver&quot;:0,&quot;super&quot;:0,&quot;super_batch_gift_num&quot;:6,&quot;super_gift_num&quot;:6,&quot;svga_block&quot;:0,&quot;tag_image&quot;:&quot;&quot;,&quot;tid&quot;:&quot;1647777083120900004&quot;,&quot;timestamp&quot;:1647777083,&quot;top_list&quot;:null,&quot;total_coin&quot;:0,&quot;uid&quot;:197750709,&quot;uname&quot;:&quot;粉色羽毛球_Official&quot;}" />
        <gift ts="0.583" user="bili_105342487" giftname="小心心" giftcount="1" raw="{&quot;action&quot;:&quot;投喂&quot;,&quot;batch_combo_id&quot;:&quot;batch:gift:combo_id:105342487:672328094:30607:1647777083.3650&quot;,&quot;batch_combo_send&quot;:null,&quot;beatId&quot;:&quot;&quot;,&quot;biz_source&quot;:&quot;live&quot;,&quot;blind_gift&quot;:null,&quot;broadcast_id&quot;:0,&quot;coin_type&quot;:&quot;silver&quot;,&quot;combo_resources_id&quot;:1,&quot;combo_send&quot;:null,&quot;combo_stay_time&quot;:3,&quot;combo_total_coin&quot;:1,&quot;crit_prob&quot;:0,&quot;demarcation&quot;:1,&quot;discount_price&quot;:0,&quot;dmscore&quot;:12,&quot;draw&quot;:0,&quot;effect&quot;:0,&quot;effect_block&quot;:1,&quot;face&quot;:&quot;http://i1.hdslb.com/bfs/face/56b139786beb080f666d283e14cd2b47755c8b93.jpg&quot;,&quot;float_sc_resource_id&quot;:0,&quot;giftId&quot;:30607,&quot;giftName&quot;:&quot;小心心&quot;,&quot;giftType&quot;:5,&quot;gold&quot;:0,&quot;guard_level&quot;:0,&quot;is_first&quot;:true,&quot;is_special_batch&quot;:0,&quot;magnification&quot;:1,&quot;medal_info&quot;:{&quot;anchor_roomid&quot;:0,&quot;anchor_uname&quot;:&quot;&quot;,&quot;guard_level&quot;:0,&quot;icon_id&quot;:0,&quot;is_lighted&quot;:1,&quot;medal_color&quot;:6126494,&quot;medal_color_border&quot;:6126494,&quot;medal_color_end&quot;:6126494,&quot;medal_color_start&quot;:6126494,&quot;medal_level&quot;:7,&quot;medal_name&quot;:&quot;莴饱了&quot;,&quot;special&quot;:&quot;&quot;,&quot;target_id&quot;:1773346},&quot;name_color&quot;:&quot;&quot;,&quot;num&quot;:1,&quot;original_gift_name&quot;:&quot;&quot;,&quot;price&quot;:0,&quot;rcost&quot;:200134843,&quot;remain&quot;:4,&quot;rnd&quot;:&quot;1647777083120700003&quot;,&quot;send_master&quot;:null,&quot;silver&quot;:0,&quot;super&quot;:0,&quot;super_batch_gift_num&quot;:1,&quot;super_gift_num&quot;:1,&quot;svga_block&quot;:0,&quot;tag_image&quot;:&quot;&quot;,&quot;tid&quot;:&quot;1647777083120700003&quot;,&quot;timestamp&quot;:1647777083,&quot;top_list&quot;:null,&quot;total_coin&quot;:0,&quot;uid&quot;:105342487,&quot;uname&quot;:&quot;bili_105342487&quot;}" />
        <d p="0.583,1,25,14893055,1647777083474,0,215087720,0" user="含着王力口乐的麦克风" raw="[[0,1,25,14893055,1647777083474,1647776382,0,&quot;47a886d6&quot;,0,0,5,&quot;#1453BAFF,#4C2263A2,#3353BAFF&quot;,0,&quot;{}&quot;,&quot;{}&quot;,{&quot;mode&quot;:0,&quot;show_player_type&quot;:0,&quot;extra&quot;:&quot;{\&quot;send_from_me\&quot;:false,\&quot;mode\&quot;:0,\&quot;color\&quot;:14893055,\&quot;dm_type\&quot;:0,\&quot;font_size\&quot;:25,\&quot;player_mode\&quot;:1,\&quot;show_player_type\&quot;:0,\&quot;content\&quot;:\&quot;快快快\&quot;,\&quot;user_hash\&quot;:\&quot;1202226902\&quot;,\&quot;emoticon_unique\&quot;:\&quot;\&quot;,\&quot;bulge_display\&quot;:0,\&quot;direction\&quot;:0,\&quot;pk_direction\&quot;:0,\&quot;quartet_direction\&quot;:0,\&quot;yeah_space_type\&quot;:\&quot;\&quot;,\&quot;yeah_space_url\&quot;:\&quot;\&quot;,\&quot;jump_to_url\&quot;:\&quot;\&quot;,\&quot;space_type\&quot;:\&quot;\&quot;,\&quot;space_url\&quot;:\&quot;\&quot;}&quot;}],&quot;快快快&quot;,[215087720,&quot;含着王力口乐的麦克风&quot;,0,0,0,10000,1,&quot;#00D1F1&quot;],[21,&quot;嘉心糖&quot;,&quot;嘉然今天吃什么&quot;,22637261,1725515,&quot;&quot;,0,6809855,1725515,5414290,3,1,672328094],[4,0,9868950,&quot;&gt;50000&quot;,0],[&quot;&quot;,&quot;&quot;],0,3,null,{&quot;ts&quot;:1647777083,&quot;ct&quot;:&quot;6D21C977&quot;},0,0,null,null,0,105]">快快快</d>
        <d p="0.583,1,25,16772431,1647777083579,0,5950232,0" user="凝华的领绳" raw="[[0,1,25,16772431,1647777083579,1647776961,0,&quot;c6fbc9f7&quot;,0,0,5,&quot;#1453BAFF,#4C2263A2,#3353BAFF&quot;,0,&quot;{}&quot;,&quot;{}&quot;,{&quot;mode&quot;:0,&quot;show_player_type&quot;:0,&quot;extra&quot;:&quot;{\&quot;send_from_me\&quot;:false,\&quot;mode\&quot;:0,\&quot;color\&quot;:16772431,\&quot;dm_type\&quot;:0,\&quot;font_size\&quot;:25,\&quot;player_mode\&quot;:1,\&quot;show_player_type\&quot;:0,\&quot;content\&quot;:\&quot;好好好\&quot;,\&quot;user_hash\&quot;:\&quot;3338390007\&quot;,\&quot;emoticon_unique\&quot;:\&quot;\&quot;,\&quot;bulge_display\&quot;:0,\&quot;direction\&quot;:0,\&quot;pk_direction\&quot;:0,\&quot;quartet_direction\&quot;:0,\&quot;yeah_space_type\&quot;:\&quot;\&quot;,\&quot;yeah_space_url\&quot;:\&quot;\&quot;,\&quot;jump_to_url\&quot;:\&quot;\&quot;,\&quot;space_type\&quot;:\&quot;\&quot;,\&quot;space_url\&quot;:\&quot;\&quot;}&quot;}],&quot;好好好&quot;,[5950232,&quot;凝华的领绳&quot;,0,0,0,10000,1,&quot;#00D1F1&quot;],[9,&quot;纯路人&quot;,&quot;猫清六合丶犬扫八方&quot;,1238503,9272486,&quot;&quot;,0,9272486,9272486,9272486,0,1,24979266],[44,0,16746162,19724,0],[&quot;&quot;,&quot;&quot;],0,3,null,{&quot;ts&quot;:1647777083,&quot;ct&quot;:&quot;7DD4EC71&quot;},0,0,null,null,0,105]">好好好</d>
        <d p="0.583,1,25,4546550,1647777083729,0,14675948,0" user="勇气花咲" raw="[[0,1,25,4546550,1647777083729,1647776966,0,&quot;ce2c1008&quot;,0,0,0,&quot;&quot;,0,&quot;{}&quot;,&quot;{}&quot;,{&quot;mode&quot;:0,&quot;show_player_type&quot;:0,&quot;extra&quot;:&quot;{\&quot;send_from_me\&quot;:false,\&quot;mode\&quot;:0,\&quot;color\&quot;:4546550,\&quot;dm_type\&quot;:0,\&quot;font_size\&quot;:25,\&quot;player_mode\&quot;:1,\&quot;show_player_type\&quot;:0,\&quot;content\&quot;:\&quot;快快快快快快\&quot;,\&quot;user_hash\&quot;:\&quot;3458994184\&quot;,\&quot;emoticon_unique\&quot;:\&quot;\&quot;,\&quot;bulge_display\&quot;:0,\&quot;direction\&quot;:0,\&quot;pk_direction\&quot;:0,\&quot;quartet_direction\&quot;:0,\&quot;yeah_space_type\&quot;:\&quot;\&quot;,\&quot;yeah_space_url\&quot;:\&quot;\&quot;,\&quot;jump_to_url\&quot;:\&quot;\&quot;,\&quot;space_type\&quot;:\&quot;\&quot;,\&quot;space_url\&quot;:\&quot;\&quot;}&quot;}],&quot;快快快快快快&quot;,[14675948,&quot;勇气花咲&quot;,0,0,0,10000,1,&quot;&quot;],[17,&quot;一个魂&quot;,&quot;A-SOUL_Official&quot;,22632157,13081892,&quot;&quot;,0,13081892,13081892,13081892,0,1,703007996],[29,0,5805790,&quot;&gt;50000&quot;,0],[&quot;&quot;,&quot;&quot;],0,0,null,{&quot;ts&quot;:1647777083,&quot;ct&quot;:&quot;D8428531&quot;},0,0,null,null,0,56]">快快快快快快</d>
        </i>
    "##;

    static BREAK_LINE: &str = r#"
    <i>
<d p="1.171,1,25,5566168,1649656743447,0,1772442517,0" user="晓小轩iAS" raw="[[0,1,25,5566168,1649656743447,1649648276,0,&quot;53372c65&quot;,0,0,0,&quot;&quot;,0,&quot;{}&quot;,&quot;{}&quot;,{&quot;mode&quot;:0,&quot;show_player_type&quot;:0,&quot;extra&quot;:&quot;{\&quot;send_from_me\&quot;:false,\&quot;mode\&quot;:0,\&quot;color\&quot;:5566168,\&quot;dm_type\&quot;:0,\&quot;font_size\&quot;:25,\&quot;player_mode\&quot;:1,\&quot;show_player_type\&quot;:0,\&quot;content\&quot;:\&quot;呵\\r呵\\r比\\r你\\r们\\r更\\r喜\\r欢\\r晚\\r晚\&quot;,\&quot;user_hash\&quot;:\&quot;1396124773\&quot;,\&quot;emoticon_unique\&quot;:\&quot;\&quot;,\&quot;bulge_display\&quot;:0,\&quot;direction\&quot;:0,\&quot;pk_direction\&quot;:0,\&quot;quartet_direction\&quot;:0,\&quot;yeah_space_type\&quot;:\&quot;\&quot;,\&quot;yeah_space_url\&quot;:\&quot;\&quot;,\&quot;jump_to_url\&quot;:\&quot;\&quot;,\&quot;space_type\&quot;:\&quot;\&quot;,\&quot;space_url\&quot;:\&quot;\&quot;}&quot;}],&quot;呵\r呵\r比\r你\r们\r更\r喜\r欢\r晚\r晚&quot;,[1772442517,&quot;晓小轩iAS&quot;,0,0,0,10000,1,&quot;&quot;],[16,&quot;顶碗人&quot;,&quot;向晚大魔王&quot;,22625025,12478086,&quot;&quot;,0,12478086,12478086,12478086,0,1,672346917],[3,0,9868950,&quot;&gt;50000&quot;,0],[&quot;&quot;,&quot;&quot;],0,0,null,{&quot;ts&quot;:1649656743,&quot;ct&quot;:&quot;4772E092&quot;},0,0,null,null,0,56]">呵
呵
比
你
们
更
喜
欢
晚
晚</d>
    </i>
    "#;

    #[test]
    fn iterator() {
        let mut parser = Parser::new(DATA.as_bytes());
        assert_eq!(
            parser.next().unwrap().unwrap(),
            Danmu {
                timeline_s: 0.581,
                content: "0-快快快".to_string(),
                r#type: DanmuType::Float,
                fontsize: 25,
                rgb: (0xe3, 0x3f, 0xff),
            }
        );
    }

    #[test]
    fn from_xml() {
        let danmu = Danmu::from_xml_p_attr("0.583,1,25,14893055,1647777083474,0,215087720,0")
            .unwrap()
            .unwrap();
        assert_eq!(
            danmu,
            Danmu {
                timeline_s: 0.583,
                content: String::new(),
                r#type: DanmuType::Float,
                fontsize: 25,
                rgb: (0xe3, 0x3f, 0xff),
            }
        );
    }

    #[test]
    fn parse_break_line() {
        let mut parser = Parser::new(BREAK_LINE.as_bytes());
        let danmu = parser.next().unwrap().unwrap();
        assert_eq!(danmu.content, "0-呵\n呵\n比\n你\n们\n更\n喜\n欢\n晚\n晚");
    }

    #[test]
    fn parse_rgb_255255255() {
        let danmu = Danmu::from_xml_p_attr(
            "1036.83700,1,25,255255255,1764772645,0,3ce09b1e,1993816477455038720,7",
        );
        let danmu = danmu.unwrap().unwrap();
        assert_eq!(danmu.rgb, (255, 255, 255));
    }
}
