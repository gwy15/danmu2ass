use crate::Danmu;

pub enum Collision {
    // 会越来越远
    Separate { closest_dis: f64 },
    // 时间够可以追上，但是时间不够
    NotEnoughTime { closest_dis: f64 },
    // 需要额外的时间才可以避免碰撞
    Collide { time_needed: f64 },
}

/// 表示一个弹幕槽位
#[derive(Debug, Clone)]
pub struct Lane {
    last_shoot_time: f64,
    last_length: u32,
}

impl Lane {
    pub fn draw(danmu: &Danmu) -> Self {
        Lane {
            last_shoot_time: danmu.timeline_s,
            last_length: danmu.length(),
        }
    }
    /// 如底部弹幕等不需要记录长度的
    pub fn draw_fixed(danmu: &Danmu) -> Self {
        Lane {
            last_shoot_time: danmu.timeline_s,
            last_length: 0,
        }
    }

    /// 这个槽位是否可以发射另外一条弹幕，返回可能的情形
    pub fn available_for(&self, other: &Danmu, config: &super::Config) -> Collision {
        #[allow(non_snake_case)]
        let T = config.duration;
        #[allow(non_snake_case)]
        let W = config.width as f64;

        // 先计算我的速度
        let t1 = self.last_shoot_time;
        let t2 = other.timeline_s;
        let l1 = self.last_length as f64;
        let l2 = other.length() as f64;

        let v1 = (W + l1) as f64 / T;
        let v2 = (W + l2) as f64 / T;

        let delta_t = t2 - t1;
        let delta_x = v1 * delta_t - l1;
        if delta_x < 0.0 {
            // 我都还没发射完呢，必定碰撞
            if l2 <= l1 {
                // 只需要把 l2 安排在 l1 之后就可以避免碰撞
                Collision::Collide {
                    time_needed: -delta_x / v1,
                }
            } else {
                // 需要延长额外的时间

                let time_needed = (t1 + T - W / v2) - t2;
                Collision::Collide { time_needed }
            }
        } else {
            // 已经发射
            let l2 = other.length() as f64;
            if l2 <= l1 {
                // 如果 l2 < l1，则它永远追不上前者，可以发射
                Collision::Separate {
                    closest_dis: delta_x,
                }
            } else {
                // 需要算追击问题了，算 l1 消失时 l2 的位置
                let pos = v2 * (T - delta_t);
                if pos < W {
                    Collision::NotEnoughTime {
                        closest_dis: W - pos,
                    }
                } else {
                    Collision::Collide {
                        time_needed: (pos - W) / v2,
                    }
                }
            }
        }
    }
}
