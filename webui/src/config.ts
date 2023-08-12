export interface IConfig {
    // 设置的版本
    version: number,

    // 屏幕宽度
    width: number,
    // 屏幕高度
    height: number,
    // 字体
    font: string,
    // 弹幕字体大小
    font_size: number,
    // 计算弹幕宽度的比例，为避免重叠可以调大这个数值
    width_ratio: number,
    // 每条弹幕之间的最小水平间距，为避免重叠可以调大这个数值。单位：像素
    horizontal_gap: number,
    // 弹幕在屏幕上的持续时间，单位为秒
    duration: number,
    // 一行弹幕所占据的高度，即“行高度/行间距”
    lane_size: number,
    // 屏幕上滚动弹幕最多高度百分比
    float_percentage: number,
    // 弹幕不透明度
    alpha: number,
    // 黑名单，需要过滤的关键词列表
    deny_list: Array<string>,
    // 描边宽度
    outline: number,
    // 是否加粗
    bold: boolean,
    // 时间轴偏移，>0 会让弹幕延后，<0 会让弹幕提前，单位为秒
    time_offset: number,
}

const LOCAL_STORAGE_KEY = 'danmu2ass-config'
const CUR_VERSION = 1;

export function load_config(): IConfig {
    // try load from localStorage
    const saved = localStorage.getItem(LOCAL_STORAGE_KEY);
    if (saved === null) {
        return make_default();
    }
    const config: any = JSON.parse(saved);
    if (config.version < CUR_VERSION) {
        const new_items: any = make_default();
        for (const key in new_items) {
            if (config[key] === undefined) {
                config[key] = new_items[key];
            }
        }
    }
    return config;

}

function make_default(): IConfig {
    return {
        version: CUR_VERSION,
        width: 1280,
        height: 720,
        font: '黑体',
        font_size: 36,
        width_ratio: 1.2,
        horizontal_gap: 20.0,
        duration: 10.0,
        lane_size: 46,
        float_percentage: 0.5,
        alpha: 0.7,
        deny_list: [],
        outline: 0.8,
        bold: true,
        time_offset: 0.0,
    }
}

export function save_config(config: IConfig) {
    localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(config));
}
