use chrono::{DateTime, Utc};
use rand::distributions::{Distribution, WeightedIndex};
use rand::{seq::SliceRandom, thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

// ==================== 数据存储结构 ====================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalancedRandData {
    pub id: String,
    pub last_updated: DateTime<Utc>,
    pub draw_counts: HashMap<u32, u32>,
    pub last_draw_round: HashMap<u32, i32>,
    pub current_round: u32,
    pub total_draws: u32,
    pub current_probabilities: HashMap<u32, f64>,

    // 配置参数
    pub min_pool_size: u32,
    pub max_gap_threshold: u32,
    pub cold_start_boost: f64,
    pub decay_factor: f64,

    // 类型识别
    pub data_type: String,

    // 2D类型参数
    pub rows: u32,
    pub cols: u32,

    // 列表类型参数
    pub numbers: Vec<u32>,

    // 范围类型参数
    pub number_range_start: u32,
    pub number_range_end: u32,

    // 黑名单/白名单
    pub blacklist: HashSet<u32>,
    pub whitelist: HashSet<u32>,
    pub whitelist_only_mode: bool,
}

// ==================== 数据管理器 ====================

pub struct BalancedRandDataManager;

impl BalancedRandDataManager {
    /// 加载所有保存的数据
    pub fn load_all_data(
        file_path: &str,
    ) -> Result<HashMap<String, BalancedRandData>, Box<dyn std::error::Error>> {
        if !Path::new(file_path).exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(file_path)?;
        let data: HashMap<String, BalancedRandData> = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// 保存所有数据
    pub fn save_all_data(
        all_data: &HashMap<String, BalancedRandData>,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(all_data)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    /// 生成唯一ID
    pub fn generate_id(data_type: &str, params: &[&str]) -> String {
        let param_string = params.join("_");
        format!("{}_{}", data_type, param_string)
    }

    /// 查找匹配的数据
    pub fn find_matching_data(
        all_data: &HashMap<String, BalancedRandData>,
        data_type: &str,
        params: &[&str],
    ) -> Option<BalancedRandData> {
        let target_id = Self::generate_id(data_type, params);

        // 精确匹配
        if let Some(data) = all_data.get(&target_id) {
            return Some(data.clone());
        }

        // 模糊匹配
        // for data in all_data.values() {
        //     if data.data_type == data_type {
        //         // TODO 实现更复杂的匹配逻辑
        //         return Some(data.clone());
        //     }
        // }

        None
    }

    /// 从指定2D配置中读取权重字典
    pub fn get_weights_by_plane_range(
        range: &[u32],
        file_path: &str,
    ) -> Result<HashMap<(u32, u32), f64>, Box<dyn std::error::Error>> {
        if range.len() != 2 {
            return Err("Plane配置参数必须包含两个元素 [rows, cols]".into());
        }

        let rows = range[0];
        let cols = range[1];

        let all_data = Self::load_all_data(file_path)?;

        // 查找匹配的2D配置数据
        for data in all_data.values() {
            if data.data_type == "BalancedRandPlane" && data.rows == rows && data.cols == cols {
                return Self::get_plane_config_weight_dict(data);
            }
        }

        Err(format!("未找到匹配的Plane配置数据: [{}, {}]", rows, cols).into())
    }

    /// 从指定2D配置中读取抽取次数字典
    pub fn get_draw_counts_by_plane_range(
        range: &[u32],
        file_path: &str,
    ) -> Result<HashMap<(u32, u32), u32>, Box<dyn std::error::Error>> {
        if range.len() != 2 {
            return Err("Plane配置参数必须包含两个元素 [rows, cols]".into());
        }

        let rows = range[0];
        let cols = range[1];

        let all_data = Self::load_all_data(file_path)?;

        // 查找匹配的2D配置数据
        for data in all_data.values() {
            if data.data_type == "BalancedRandPlane" && data.rows == rows && data.cols == cols {
                return Self::get_plane_config_draw_counts_dict(data);
            }
        }

        Err(format!("未找到匹配的Plane配置数据: [{}, {}]", rows, cols).into())
    }

    /// 从2D配置数据中提取权重字典
    fn get_plane_config_weight_dict(
        data: &BalancedRandData,
    ) -> Result<HashMap<(u32, u32), f64>, Box<dyn std::error::Error>> {
        if data.data_type != "BalancedRandPlane" {
            return Err("数据类型必须是BalancedRandPlane".into());
        }

        let rows = data.rows;
        let cols = data.cols;
        let total_positions = rows * cols;

        let mut weights = HashMap::new();

        // 对于2D数据，按位置顺序提取权重（行优先）
        for i in 0..total_positions {
            let row = i / cols + 1; // 1-based 行
            let col = i % cols + 1; // 1-based 列

            // 检查该位置是否在黑名单中
            let is_blacklisted = data.blacklist.contains(&i);

            if let Some(&weight) = data.current_probabilities.get(&i) {
                // 如果在黑名单中，权重设为0，否则使用实际权重
                weights.insert((row, col), if is_blacklisted { 0.0 } else { weight });
            } else if is_blacklisted {
                // 如果在黑名单中但权重数据不存在，设为0
                weights.insert((row, col), 0.0);
            } else {
                // 不在黑名单中且权重数据不存在，设为0
                weights.insert((row, col), 0.0);
            }
        }

        Ok(weights)
    }

    /// 从2D配置数据中提取抽取次数字典
    fn get_plane_config_draw_counts_dict(
        data: &BalancedRandData,
    ) -> Result<HashMap<(u32, u32), u32>, Box<dyn std::error::Error>> {
        if data.data_type != "BalancedRandPlane" {
            return Err("数据类型必须是BalancedRandPlane".into());
        }

        let rows = data.rows;
        let cols = data.cols;
        let total_positions = rows * cols;

        let mut draw_counts = HashMap::new();

        // 对于2D数据，按位置顺序提取抽取次数（行优先）
        for i in 0..total_positions {
            let row = i / cols + 1; // 1-based 行
            let col = i % cols + 1; // 1-based 列

            // 检查该位置是否在黑名单中
            let is_blacklisted = data.blacklist.contains(&i);

            if let Some(&count) = data.draw_counts.get(&i) {
                // 如果在黑名单中，抽取次数设为0，否则使用实际值
                draw_counts.insert((row, col), if is_blacklisted { 0 } else { count });
            } else if is_blacklisted {
                // 如果在黑名单中但抽取次数数据不存在，设为0
                draw_counts.insert((row, col), 0);
            } else {
                // 不在黑名单中且抽取次数数据不存在，设为0
                draw_counts.insert((row, col), 0);
            }
        }

        Ok(draw_counts)
    }
}

// ==================== 平衡随机抽取类 ====================

pub struct BalancedRand {
    // 内部数据结构
    draw_counts: HashMap<u32, u32>,
    last_draw_round: HashMap<u32, i32>,
    all_numbers: Vec<u32>,
    candidate_pool: Vec<u32>,

    // 配置参数
    current_round: u32,
    min_pool_size: u32,
    max_gap_threshold: u32,
    cold_start_boost: f64,
    decay_factor: f64,

    // 统计信息
    total_draws: u32,
    current_probabilities: HashMap<u32, f64>,

    // 数据标识
    data_id: String,
    data_type: String,

    // 构造函数参数
    number_range_start: u32,
    number_range_end: u32,
    numbers_list: Option<Vec<u32>>,

    // 黑名单/白名单
    blacklist: HashSet<u32>,
    whitelist: HashSet<u32>,
    whitelist_only_mode: bool,
}

impl BalancedRand {
    /// 构造函数（学号范围）
    pub fn new_from_range(
        number_range_start: u32,
        number_range_end: u32,
        min_pool_size: u32,
        max_gap_threshold: u32,
        cold_start_boost: f64,
        decay_factor: f64,
        load_data: bool,
    ) -> Result<Self, String> {
        if number_range_start > number_range_end {
            return Err("起始值不能大于结束值".to_string());
        }

        if min_pool_size == 0 {
            return Err("最小候选池大小必须大于0".to_string());
        }

        // 生成学号列表
        let all_numbers: Vec<u32> = (number_range_start..=number_range_end).collect();

        // 初始化数据结构
        let draw_counts: HashMap<u32, u32> = all_numbers.iter().map(|&n| (n, 0)).collect();
        let last_draw_round: HashMap<u32, i32> = all_numbers.iter().map(|&n| (n, -1)).collect();

        // 生成数据ID
        let params = vec![
            number_range_start.to_string(),
            number_range_end.to_string(),
            min_pool_size.to_string(),
            max_gap_threshold.to_string(),
            cold_start_boost.to_string(),
            decay_factor.to_string(),
        ];
        let param_strings: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
        let data_id = BalancedRandDataManager::generate_id("BalancedRand_Range", &param_strings);

        let mut instance = Self {
            draw_counts,
            last_draw_round,
            all_numbers: all_numbers.clone(),
            candidate_pool: Vec::new(),
            current_round: 0,
            min_pool_size,
            max_gap_threshold,
            cold_start_boost,
            decay_factor,
            total_draws: 0,
            current_probabilities: HashMap::new(),
            data_id,
            data_type: "BalancedRand_Range".to_string(),
            number_range_start,
            number_range_end,
            numbers_list: None,
            blacklist: HashSet::new(),
            whitelist: HashSet::new(),
            whitelist_only_mode: false,
        };

        // 初始化候选池
        instance.update_candidate_pool();

        // 加载历史数据
        if load_data {
            if let Err(e) = instance.load_data("balanced_rand_data.json") {
                eprintln!("加载数据失败: {}", e);
            }
        }

        Ok(instance)
    }

    /// 构造函数（学号列表）
    pub fn new_from_list(
        numbers: &[u32],
        min_pool_size: u32,
        max_gap_threshold: u32,
        cold_start_boost: f64,
        decay_factor: f64,
        load_data: bool,
    ) -> Result<Self, String> {
        if numbers.is_empty() {
            return Err("学号列表不能为空".to_string());
        }

        // 去重
        let mut all_numbers: Vec<u32> = numbers.to_vec();
        all_numbers.sort_unstable();
        all_numbers.dedup();

        // 初始化数据结构
        let draw_counts: HashMap<u32, u32> = all_numbers.iter().map(|&n| (n, 0)).collect();
        let last_draw_round: HashMap<u32, i32> = all_numbers.iter().map(|&n| (n, -1)).collect();

        // 生成数据ID（使用前10个学号）
        let numbers_str = if all_numbers.len() > 10 {
            all_numbers[..10]
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(",")
        } else {
            all_numbers
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(",")
        };

        let params = vec![
            numbers_str,
            min_pool_size.to_string(),
            max_gap_threshold.to_string(),
            cold_start_boost.to_string(),
            decay_factor.to_string(),
        ];
        let param_strings: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
        let data_id = BalancedRandDataManager::generate_id("BalancedRand_List", &param_strings);

        let mut instance = Self {
            draw_counts,
            last_draw_round,
            all_numbers: all_numbers.clone(),
            candidate_pool: Vec::new(),
            current_round: 0,
            min_pool_size,
            max_gap_threshold,
            cold_start_boost,
            decay_factor,
            total_draws: 0,
            current_probabilities: HashMap::new(),
            data_id,
            data_type: "BalancedRand_List".to_string(),
            number_range_start: 0,
            number_range_end: 0,
            numbers_list: Some(all_numbers.clone()),
            blacklist: HashSet::new(),
            whitelist: HashSet::new(),
            whitelist_only_mode: false,
        };

        instance.update_candidate_pool();

        if load_data {
            if let Err(e) = instance.load_data("balanced_rand_data.json") {
                eprintln!("加载数据失败: {}", e);
            }
        }

        Ok(instance)
    }

    /// 加载数据
    pub fn load_data(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let all_data = BalancedRandDataManager::load_all_data(file_path)?;

        if let Some(saved_data) = all_data.get(&self.data_id) {
            self.apply_saved_data(saved_data);
            println!("已加载数据: {}", self.data_id);
        }

        Ok(())
    }

    /// 应用保存的数据
    fn apply_saved_data(&mut self, saved_data: &BalancedRandData) {
        // 加载抽取次数
        for (&key, &value) in &saved_data.draw_counts {
            if self.draw_counts.contains_key(&key) {
                self.draw_counts.insert(key, value);
            }
        }

        // 加载最后抽取轮次
        for (&key, &value) in &saved_data.last_draw_round {
            if self.last_draw_round.contains_key(&key) {
                self.last_draw_round.insert(key, value);
            }
        }

        self.current_round = saved_data.current_round;
        self.total_draws = saved_data.total_draws;

        // 加载概率
        for (&key, &value) in &saved_data.current_probabilities {
            if self.current_probabilities.contains_key(&key) {
                self.current_probabilities.insert(key, value);
            }
        }

        // 更新配置参数
        self.min_pool_size = saved_data.min_pool_size;
        self.max_gap_threshold = saved_data.max_gap_threshold;
        self.cold_start_boost = saved_data.cold_start_boost;
        self.decay_factor = saved_data.decay_factor;

        // 加载黑名单/白名单
        self.blacklist = saved_data.blacklist.clone();
        self.whitelist = saved_data.whitelist.clone();
        self.whitelist_only_mode = saved_data.whitelist_only_mode;

        // 验证黑名单和白名单
        self.validate_blacklist();
        self.validate_whitelist();

        // 更新候选池
        self.update_candidate_pool();
    }

    /// 保存数据
    pub fn save_data(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut all_data = BalancedRandDataManager::load_all_data(file_path).unwrap_or_default();

        let data = BalancedRandData {
            id: self.data_id.clone(),
            last_updated: Utc::now(),
            draw_counts: self.draw_counts.clone(),
            last_draw_round: self.last_draw_round.clone(),
            current_round: self.current_round,
            total_draws: self.total_draws,
            current_probabilities: self.current_probabilities.clone(),
            min_pool_size: self.min_pool_size,
            max_gap_threshold: self.max_gap_threshold,
            cold_start_boost: self.cold_start_boost,
            decay_factor: self.decay_factor,
            data_type: self.data_type.clone(),
            rows: 0,
            cols: 0,
            numbers: self.numbers_list.clone().unwrap_or_default(),
            number_range_start: self.number_range_start,
            number_range_end: self.number_range_end,
            blacklist: self.blacklist.clone(),
            whitelist: self.whitelist.clone(),
            whitelist_only_mode: self.whitelist_only_mode,
        };

        all_data.insert(self.data_id.clone(), data);
        BalancedRandDataManager::save_all_data(&all_data, file_path)?;

        println!("已保存数据: {}", self.data_id);
        Ok(())
    }

    // ==================== 黑名单/白名单功能 ====================

    /// 设置黑名单
    pub fn set_blacklist(&mut self, numbers: &[u32]) {
        self.blacklist.clear();
        for &number in numbers {
            if self.all_numbers.contains(&number) {
                self.blacklist.insert(number);
            }
        }
        self.validate_blacklist();
        self.update_candidate_pool();
    }

    /// 添加学号到黑名单
    pub fn add_to_blacklist(&mut self, numbers: &[u32]) {
        for &number in numbers {
            if self.all_numbers.contains(&number) && !self.blacklist.contains(&number) {
                self.blacklist.insert(number);
            }
        }
        self.validate_blacklist();
        self.update_candidate_pool();
    }

    /// 从黑名单中移除学号
    pub fn remove_from_blacklist(&mut self, numbers: &[u32]) {
        for &number in numbers {
            self.blacklist.remove(&number);
        }
        self.update_candidate_pool();
    }

    /// 清除所有黑名单
    pub fn clear_blacklist(&mut self) {
        self.blacklist.clear();
        self.update_candidate_pool();
    }

    /// 获取当前黑名单
    pub fn get_blacklist(&self) -> Vec<u32> {
        self.blacklist.iter().copied().collect()
    }

    /// 检查学号是否在黑名单中
    pub fn is_in_blacklist(&self, number: u32) -> bool {
        self.blacklist.contains(&number)
    }

    /// 设置白名单
    pub fn set_whitelist(&mut self, numbers: &[u32]) {
        self.whitelist.clear();
        for &number in numbers {
            self.whitelist.insert(number);
        }
        self.validate_whitelist();
        self.update_candidate_pool();
    }

    /// 添加学号到白名单
    pub fn add_to_whitelist(&mut self, numbers: &[u32]) {
        for &number in numbers {
            if !self.whitelist.contains(&number) {
                self.whitelist.insert(number);
            }
        }
        self.validate_whitelist();
        self.update_candidate_pool();
    }

    /// 从白名单中移除学号
    pub fn remove_from_whitelist(&mut self, numbers: &[u32]) {
        for &number in numbers {
            self.whitelist.remove(&number);
        }
        self.update_candidate_pool();
    }

    /// 清除所有白名单
    pub fn clear_whitelist(&mut self) {
        self.whitelist.clear();
        self.update_candidate_pool();
    }

    /// 获取当前白名单
    pub fn get_whitelist(&self) -> Vec<u32> {
        self.whitelist.iter().copied().collect()
    }

    /// 检查学号是否在白名单中
    pub fn is_in_whitelist(&self, number: u32) -> bool {
        self.whitelist.contains(&number)
    }

    /// 设置白名单模式
    pub fn set_whitelist_only_mode(&mut self, whitelist_only: bool) {
        self.whitelist_only_mode = whitelist_only;
        self.update_candidate_pool();
    }

    /// 获取当前是否处于白名单模式
    pub fn get_whitelist_only_mode(&self) -> bool {
        self.whitelist_only_mode
    }

    /// 验证黑名单的合法性
    fn validate_blacklist(&mut self) {
        let mut to_remove = Vec::new();
        for &number in &self.blacklist {
            if !self.all_numbers.contains(&number) {
                to_remove.push(number);
            }
        }
        for number in to_remove {
            self.blacklist.remove(&number);
        }
    }

    /// 验证白名单的合法性
    fn validate_whitelist(&self) {
        // 白名单不需要验证，可以包含不在all_numbers中的学号
    }

    // ==================== 核心功能 ====================

    /// 抽取一个学号
    pub fn draw(&mut self, auto_save: bool) -> Result<u32, String> {
        if self.candidate_pool.is_empty() {
            // 如果候选池为空，重置所有抽取次数
            self.reset_draw_counts();
        }

        self.current_round += 1;

        // 计算权重
        let weights = self.calculate_weights();

        // 根据权重进行随机抽取
        let selected_number = self.weighted_random_select(&weights)?;

        // 更新抽取记录
        let count = self.draw_counts.entry(selected_number).or_insert(0);
        *count += 1;

        self.last_draw_round
            .insert(selected_number, self.current_round as i32);
        self.total_draws += 1;

        // 更新候选池和概率
        self.update_candidate_pool();
        self.update_probabilities();

        // 自动保存数据
        if auto_save {
            if let Err(e) = self.save_data("balanced_rand_data.json") {
                eprintln!("保存数据失败: {}", e);
            }
        }

        Ok(selected_number)
    }

    /// 批量抽取多个学号
    pub fn draw_multiple(&mut self, count: u32, auto_save: bool) -> Result<Vec<u32>, String> {
        if count == 0 {
            return Err("抽取数量必须大于0".to_string());
        }

        if count > self.candidate_pool.len() as u32 {
            return Err(format!(
                "抽取数量不能超过候选池大小({})",
                self.candidate_pool.len()
            ));
        }

        let mut results = Vec::new();

        for i in 0..count {
            // 只在最后一次抽取后保存
            let save = (i == count - 1) && auto_save;
            let result = self.draw(save)?;
            results.push(result);
        }

        Ok(results)
    }

    /// 重置所有抽取次数
    pub fn reset_draw_counts(&mut self) {
        // 重置原始学号范围的抽取次数
        for &number in &self.all_numbers {
            self.draw_counts.insert(number, 0);
            self.last_draw_round.insert(number, -1);
        }

        // 重置白名单学号的抽取次数
        for &number in &self.whitelist {
            self.draw_counts.insert(number, 0);
            self.last_draw_round.insert(number, -1);
        }

        self.total_draws = 0;
        self.current_round = 0;
        self.update_candidate_pool();
    }

    /// 获取统计数据
    pub fn get_statistics(&self) -> Vec<(u32, u32)> {
        let mut all_numbers = self.all_numbers.clone();
        for &number in &self.whitelist {
            if !all_numbers.contains(&number) {
                all_numbers.push(number);
            }
        }
        all_numbers.sort_unstable();

        all_numbers
            .iter()
            .map(|&n| {
                let count = self.draw_counts.get(&n).copied().unwrap_or(0);
                (n, count)
            })
            .collect()
    }

    /// 获取概率数据
    pub fn get_probabilities(&self) -> Vec<(u32, f64)> {
        let mut all_numbers = self.all_numbers.clone();
        for &number in &self.whitelist {
            if !all_numbers.contains(&number) {
                all_numbers.push(number);
            }
        }
        all_numbers.sort_unstable();

        all_numbers
            .iter()
            .map(|&n| {
                let prob = self.current_probabilities.get(&n).copied().unwrap_or(0.0);
                (n, prob)
            })
            .collect()
    }

    // ==================== 私有方法 ====================

    /// 更新候选池
    fn update_candidate_pool(&mut self) {
        let mut candidates = Vec::new();

        if self.whitelist_only_mode {
            // 白名单模式：只从白名单中抽取
            candidates = self.whitelist.iter().copied().collect();
        } else {
            // 正常模式：从原始学号范围中筛选
            let average = self.get_average_draw_count();

            // 平均值过滤
            candidates = self
                .all_numbers
                .iter()
                .filter(|&&n| {
                    let count = self.draw_counts.get(&n).copied().unwrap_or(0);
                    count as f64 <= average.ceil()
                })
                .copied()
                .collect();

            // 最大差距保护
            if self.get_max_draw_count_gap() > self.max_gap_threshold {
                // 排除极值
                let max_count = self.draw_counts.values().max().copied().unwrap_or(0);
                let min_count = self.draw_counts.values().min().copied().unwrap_or(0);

                candidates.retain(|&n| {
                    let count = self.draw_counts.get(&n).copied().unwrap_or(0);
                    count != max_count && count != min_count
                });

                // 重新计算排除极值后的平均值
                if !candidates.is_empty() {
                    let new_average: f64 = candidates
                        .iter()
                        .map(|&n| self.draw_counts.get(&n).copied().unwrap_or(0) as f64)
                        .sum::<f64>()
                        / candidates.len() as f64;

                    candidates.retain(|&n| {
                        let count = self.draw_counts.get(&n).copied().unwrap_or(0);
                        count as f64 <= new_average.ceil()
                    });
                }
            }

            // 加入白名单中的额外学号
            for &number in &self.whitelist {
                if !candidates.contains(&number) {
                    candidates.push(number);
                }
            }
        }

        // 移除黑名单中的学号
        candidates.retain(|&n| !self.blacklist.contains(&n));

        // 候选池大小检查
        if candidates.len() < self.min_pool_size as usize {
            // 添加抽取次数较低的成员
            let mut all_available: Vec<u32> = self.all_numbers.clone();
            for &number in &self.whitelist {
                if !all_available.contains(&number) {
                    all_available.push(number);
                }
            }

            all_available.retain(|&n| !self.blacklist.contains(&n) && !candidates.contains(&n));

            // 按抽取次数和最后抽取轮次排序
            all_available.sort_by_key(|&n| {
                let count = self.draw_counts.get(&n).copied().unwrap_or(0);
                let round = self.last_draw_round.get(&n).copied().unwrap_or(i32::MAX);
                (count, round)
            });

            let needed = self.min_pool_size as usize - candidates.len();
            for &number in all_available.iter().take(needed) {
                if !candidates.contains(&number) {
                    candidates.push(number);
                }
            }
        }

        self.candidate_pool = candidates;
    }

    /// 计算权重
    fn calculate_weights(&self) -> HashMap<u32, f64> {
        let mut weights = HashMap::new();

        for &number in &self.candidate_pool {
            if self.blacklist.contains(&number) {
                continue;
            }

            let mut weight = 1.0;

            // 获取抽取次数
            let draw_count = self.draw_counts.get(&number).copied().unwrap_or(0);

            // 避免重复抽取
            weight *= self.decay_factor.powi(draw_count as i32);

            // 长期未被抽中的成员权重提升
            let last_round = self.last_draw_round.get(&number).copied().unwrap_or(-1);

            if last_round < 0 {
                // 从未被抽中
                weight *= self.cold_start_boost;
            } else {
                let rounds_since_last_draw = self.current_round as i32 - last_round;
                let active_numbers_count = self.all_numbers.len()
                    + self
                        .whitelist
                        .iter()
                        .filter(|&&n| !self.all_numbers.contains(&n))
                        .count();

                if rounds_since_last_draw > active_numbers_count as i32 / 2 {
                    weight *= 1.0 + (rounds_since_last_draw as f64 + 1.0).ln() / 10.0;
                }
            }

            // 抽取次数倒数权重
            weight *= 1.0 / (draw_count as f64 + 1.0);

            // 白名单权重提升
            if !self.all_numbers.contains(&number) && self.whitelist.contains(&number) {
                weight *= self.cold_start_boost;
            }

            // 保证最小权重
            weights.insert(number, weight.max(0.01));
        }

        weights
    }

    /// 根据权重进行随机选择
    fn weighted_random_select(&self, weights: &HashMap<u32, f64>) -> Result<u32, String> {
        if weights.is_empty() {
            return Err("权重字典为空".to_string());
        }

        let (numbers, weight_values): (Vec<u32>, Vec<f64>) =
            weights.iter().map(|(&num, &weight)| (num, weight)).unzip();

        match WeightedIndex::new(&weight_values) {
            Ok(dist) => {
                let mut rng = thread_rng();
                let idx = dist.sample(&mut rng);
                Ok(numbers[idx])
            }
            Err(_) => {
                // 如果权重有问题，使用均匀随机
                self.candidate_pool
                    .choose(&mut thread_rng())
                    .copied()
                    .ok_or_else(|| "无法从候选池中选择".to_string())
            }
        }
    }

    /// 更新概率信息
    fn update_probabilities(&mut self) {
        self.current_probabilities.clear();

        if self.candidate_pool.is_empty() {
            return;
        }

        let weights = self.calculate_weights();
        let total_weight: f64 = weights.values().sum();

        for (&number, &weight) in &weights {
            self.current_probabilities
                .insert(number, weight / total_weight);
        }

        // 为不在候选池中的成员设置概率为0
        let mut all_active = self.all_numbers.clone();
        for &number in &self.whitelist {
            if !all_active.contains(&number) {
                all_active.push(number);
            }
        }

        for &number in &all_active {
            if !self.candidate_pool.contains(&number) {
                self.current_probabilities.insert(number, 0.0);
            }
        }
    }

    /// 获取平均抽取次数
    fn get_average_draw_count(&self) -> f64 {
        let mut all_active = self.all_numbers.clone();
        for &number in &self.whitelist {
            if !all_active.contains(&number) {
                all_active.push(number);
            }
        }

        if all_active.is_empty() {
            return 0.0;
        }

        let total: u32 = all_active
            .iter()
            .map(|&n| self.draw_counts.get(&n).copied().unwrap_or(0))
            .sum();

        total as f64 / all_active.len() as f64
    }

    /// 获取最大抽取次数差距
    fn get_max_draw_count_gap(&self) -> u32 {
        let mut all_active = self.all_numbers.clone();
        for &number in &self.whitelist {
            if !all_active.contains(&number) {
                all_active.push(number);
            }
        }

        if all_active.is_empty() {
            return 0;
        }

        let active_draw_counts: Vec<u32> = all_active
            .iter()
            .map(|&n| self.draw_counts.get(&n).copied().unwrap_or(0))
            .collect();

        let max_count = active_draw_counts.iter().max().copied().unwrap_or(0);
        let min_count = active_draw_counts.iter().min().copied().unwrap_or(0);

        max_count - min_count
    }
}

// ==================== 2D版本 ====================

pub struct BalancedRandPlane {
    balanced_rand: BalancedRand,
    rows: u32,
    cols: u32,
    data_id_plane: String,
}

impl BalancedRandPlane {
    /// 构造函数
    pub fn new(
        rows: u32,
        cols: u32,
        min_pool_size: u32,
        max_gap_threshold: u32,
        cold_start_boost: f64,
        decay_factor: f64,
        load_data: bool,
    ) -> Result<Self, String> {
        // 使用基类构造函数
        let balanced_rand = BalancedRand::new_from_range(
            0,
            rows * cols - 1,
            min_pool_size,
            max_gap_threshold,
            cold_start_boost,
            decay_factor,
            false,
        )?;

        // 生成2D专用的数据ID
        let params = vec![
            rows.to_string(),
            cols.to_string(),
            min_pool_size.to_string(),
            max_gap_threshold.to_string(),
            cold_start_boost.to_string(),
            decay_factor.to_string(),
        ];
        let param_strings: Vec<&str> = params.iter().map(|s| s.as_str()).collect();
        let data_id_plane =
            BalancedRandDataManager::generate_id("BalancedRandPlane", &param_strings);

        let mut instance = Self {
            balanced_rand,
            rows,
            cols,
            data_id_plane,
        };

        // 加载历史数据
        if load_data {
            if let Err(e) = instance.load_data("balanced_rand_data.json") {
                eprintln!("加载Plane数据失败: {}", e);
            }
        }

        Ok(instance)
    }

    /// 抽取一个位置
    pub fn draw_position(&mut self, auto_save: bool) -> Result<(u32, u32), String> {
        let number = self.balanced_rand.draw(auto_save)?;
        // 将0-based学号转换为1-based行列
        let zero_based_number = number;
        let row = zero_based_number / self.cols + 1;
        let col = zero_based_number % self.cols + 1;
        Ok((row, col))
    }

    /// 批量抽取多个位置
    pub fn draw_multiple_positions(
        &mut self,
        count: u32,
        auto_save: bool,
    ) -> Result<Vec<(u32, u32)>, String> {
        let numbers = self.balanced_rand.draw_multiple(count, auto_save)?;

        let positions: Vec<(u32, u32)> = numbers
            .iter()
            .map(|&n| {
                let zero_based_number = n;
                let row = zero_based_number / self.cols + 1;
                let col = zero_based_number % self.cols + 1;
                (row, col)
            })
            .collect();

        Ok(positions)
    }

    /// 获取位置概率字典
    pub fn get_position_probabilities_dict(&self) -> HashMap<(u32, u32), f64> {
        let mut probabilities = HashMap::new();

        // 遍历所有位置
        for row in 1..=self.rows {
            for col in 1..=self.cols {
                // 计算对应的学号 (0-based)
                let number = (row - 1) * self.cols + (col - 1);

                // 检查是否在黑名单中
                let is_blacklisted = self.balanced_rand.is_in_blacklist(number);

                // 获取概率值
                let prob = if is_blacklisted {
                    // 黑名单中的位置概率为0
                    0.0
                } else {
                    // 获取实际概率
                    self.balanced_rand
                        .current_probabilities
                        .get(&number)
                        .copied()
                        .unwrap_or(0.0)
                };

                probabilities.insert((row, col), prob);
            }
        }

        probabilities
    }

    /// 获取位置抽取次数字典
    pub fn get_position_draw_counts_dict(&self) -> HashMap<(u32, u32), u32> {
        let mut draw_counts = HashMap::new();

        // 遍历所有位置
        for row in 1..=self.rows {
            for col in 1..=self.cols {
                // 计算对应的学号 (0-based)
                let number = (row - 1) * self.cols + (col - 1);

                // 检查是否在黑名单中
                let is_blacklisted = self.balanced_rand.is_in_blacklist(number);

                // 获取抽取次数
                let count = if is_blacklisted {
                    // 黑名单中的位置抽取次数为0
                    0
                } else {
                    // 获取实际抽取次数
                    self.balanced_rand
                        .draw_counts
                        .get(&number)
                        .copied()
                        .unwrap_or(0)
                };

                draw_counts.insert((row, col), count);
            }
        }

        draw_counts
    }

    /// 获取位置统计数据字典（包含抽取次数、概率等）
    pub fn get_position_statistics_dict(&self) -> HashMap<(u32, u32), (u32, f64, i32)> {
        let mut statistics = HashMap::new();

        // 遍历所有位置
        for row in 1..=self.rows {
            for col in 1..=self.cols {
                // 计算对应的学号 (0-based)
                let number = (row - 1) * self.cols + (col - 1);

                // 检查是否在黑名单中
                let is_blacklisted = self.balanced_rand.is_in_blacklist(number);

                // 获取各项统计数据
                let (draw_count, probability, last_draw_round) = if is_blacklisted {
                    // 黑名单中的位置：抽取次数为0，概率为0，最后抽取轮次为-1
                    (0, 0.0, -1)
                } else {
                    // 获取实际数据
                    let draw_count = self
                        .balanced_rand
                        .draw_counts
                        .get(&number)
                        .copied()
                        .unwrap_or(0);

                    let probability = self
                        .balanced_rand
                        .current_probabilities
                        .get(&number)
                        .copied()
                        .unwrap_or(0.0);

                    let last_draw_round = self
                        .balanced_rand
                        .last_draw_round
                        .get(&number)
                        .copied()
                        .unwrap_or(-1);

                    (draw_count, probability, last_draw_round)
                };

                statistics.insert((row, col), (draw_count, probability, last_draw_round));
            }
        }

        statistics
    }

    /// 获取平均抽取次数
    pub fn get_average_draw_count(&self) -> f64 {
        self.balanced_rand.get_average_draw_count()
    }

    /// 获取最大抽取次数差距
    pub fn get_max_draw_count_gap(&self) -> u32 {
        self.balanced_rand.get_max_draw_count_gap()
    }

    /// 加载数据
    pub fn load_data(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let all_data = BalancedRandDataManager::load_all_data(file_path)?;

        // 优先使用2D专用ID
        if let Some(saved_data) = all_data.get(&self.data_id_plane) {
            self.apply_saved_data(saved_data);
            println!("已加载Plane数据: {}", self.data_id_plane);
        }

        Ok(())
    }

    /// 应用保存的数据
    fn apply_saved_data(&mut self, saved_data: &BalancedRandData) {
        // 可以在这里添加2D特有的处理逻辑
        self.balanced_rand.apply_saved_data(saved_data);
    }

    /// 保存数据
    pub fn save_data(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut all_data = BalancedRandDataManager::load_all_data(file_path).unwrap_or_default();

        // 获取基类的数据
        let data = BalancedRandData {
            id: self.data_id_plane.clone(),
            last_updated: Utc::now(),
            draw_counts: self.balanced_rand.draw_counts.clone(),
            last_draw_round: self.balanced_rand.last_draw_round.clone(),
            current_round: self.balanced_rand.current_round,
            total_draws: self.balanced_rand.total_draws,
            current_probabilities: self.balanced_rand.current_probabilities.clone(),
            min_pool_size: self.balanced_rand.min_pool_size,
            max_gap_threshold: self.balanced_rand.max_gap_threshold,
            cold_start_boost: self.balanced_rand.cold_start_boost,
            decay_factor: self.balanced_rand.decay_factor,
            data_type: "BalancedRandPlane".to_string(),
            rows: self.rows,
            cols: self.cols,
            numbers: Vec::new(),
            number_range_start: 0,
            number_range_end: self.rows * self.cols - 1,
            blacklist: self.balanced_rand.blacklist.clone(),
            whitelist: self.balanced_rand.whitelist.clone(),
            whitelist_only_mode: self.balanced_rand.whitelist_only_mode,
        };

        all_data.insert(self.data_id_plane.clone(), data);
        BalancedRandDataManager::save_all_data(&all_data, file_path)?;

        println!("已保存Plane数据: {}", self.data_id_plane);
        Ok(())
    }

    // ==================== 2D专用的黑名单/白名单功能 ====================

    /// 设置黑名单位置
    pub fn set_blacklist_positions(&mut self, positions: &[(u32, u32)]) {
        let numbers: Vec<u32> = positions
            .iter()
            .filter(|&&(row, col)| row >= 1 && row <= self.rows && col >= 1 && col <= self.cols)
            .map(|&(row, col)| (row - 1) * self.cols + (col - 1))
            .collect();

        self.balanced_rand.set_blacklist(&numbers);
    }

    /// 添加位置到黑名单
    pub fn add_to_blacklist_positions(&mut self, positions: &[(u32, u32)]) {
        let numbers: Vec<u32> = positions
            .iter()
            .filter(|&&(row, col)| row >= 1 && row <= self.rows && col >= 1 && col <= self.cols)
            .map(|&(row, col)| (row - 1) * self.cols + (col - 1))
            .collect();

        self.balanced_rand.add_to_blacklist(&numbers);
    }

    /// 从黑名单中移除位置
    pub fn remove_from_blacklist_positions(&mut self, positions: &[(u32, u32)]) {
        let numbers: Vec<u32> = positions
            .iter()
            .filter(|&&(row, col)| row >= 1 && row <= self.rows && col >= 1 && col <= self.cols)
            .map(|&(row, col)| (row - 1) * self.cols + (col - 1))
            .collect();

        self.balanced_rand.remove_from_blacklist(&numbers);
    }

    /// 检查位置是否在黑名单中
    pub fn is_position_in_blacklist(&self, row: u32, col: u32) -> bool {
        if row < 1 || row > self.rows || col < 1 || col > self.cols {
            return false;
        }

        let number = (row - 1) * self.cols + (col - 1);
        self.balanced_rand.is_in_blacklist(number)
    }

    /// 设置白名单位置
    pub fn set_whitelist_positions(&mut self, positions: &[(u32, u32)]) {
        let numbers: Vec<u32> = positions
            .iter()
            .map(|&(row, col)| (row - 1) * self.cols + (col - 1))
            .collect();

        self.balanced_rand.set_whitelist(&numbers);
    }

    /// 添加位置到白名单
    pub fn add_to_whitelist_positions(&mut self, positions: &[(u32, u32)]) {
        let numbers: Vec<u32> = positions
            .iter()
            .map(|&(row, col)| (row - 1) * self.cols + (col - 1))
            .collect();

        self.balanced_rand.add_to_whitelist(&numbers);
    }

    /// 从白名单中移除位置
    pub fn remove_from_whitelist_positions(&mut self, positions: &[(u32, u32)]) {
        let numbers: Vec<u32> = positions
            .iter()
            .map(|&(row, col)| (row - 1) * self.cols + (col - 1))
            .collect();

        self.balanced_rand.remove_from_whitelist(&numbers);
    }

    /// 检查位置是否在白名单中
    pub fn is_position_in_whitelist(&self, row: u32, col: u32) -> bool {
        let number = (row - 1) * self.cols + (col - 1);
        self.balanced_rand.is_in_whitelist(number)
    }

    /// 设置白名单模式
    pub fn set_whitelist_only_mode(&mut self, whitelist_only: bool) {
        self.balanced_rand.set_whitelist_only_mode(whitelist_only);
    }

    /// 获取当前是否处于白名单模式
    pub fn get_whitelist_only_mode(&self) -> bool {
        self.balanced_rand.get_whitelist_only_mode()
    }
}

// ==================== 示例用法 ====================

fn main() {
    // 示例: 使用2D版本
    let mut plane = BalancedRandPlane::new(
        3, 4,    // 3行4列
        3,    // 最小候选池大小
        5,    // 最大差距阈值
        2.0,  // 冷启动提升系数
        0.7,  // 衰减因子
        true, // 加载历史数据
    )
    .expect("创建BalancedRandPlane失败");

    // 设置一些黑名单位置
    plane.set_blacklist_positions(&[(1, 1), (2, 3)]);

    // 抽取几个位置
    println!("抽取位置:");
    for _ in 0..5 {
        match plane.draw_position(true) {
            Ok((row, col)) => println!("  - 第{}行, 第{}列", row, col),
            Err(e) => eprintln!("抽取失败: {}", e),
        }
    }

    // 获取位置概率字典
    println!("\n位置概率字典:");
    let probabilities = plane.get_position_probabilities_dict();
    for ((row, col), prob) in &probabilities {
        println!("  - ({}, {}): {:.3}", row, col, prob);
    }

    // 获取位置抽取次数字典
    println!("\n位置抽取次数字典:");
    let draw_counts = plane.get_position_draw_counts_dict();
    for ((row, col), count) in &draw_counts {
        println!("  - ({}, {}): {}", row, col, count);
    }

    // 获取完整统计数据字典
    println!("\n完整统计数据字典:");
    let stats = plane.get_position_statistics_dict();
    for ((row, col), (count, prob, last_round)) in &stats {
        println!(
            "  - ({}, {}): 抽取次数={}, 概率={:.3}, 最后抽取轮次={}",
            row, col, count, prob, last_round
        );
    }

    // 检查黑名单位置
    println!("\n黑名单检查:");
    println!(
        "  - (1,1) 是否在黑名单中: {}",
        plane.is_position_in_blacklist(1, 1)
    );
    println!(
        "  - (2,3) 是否在黑名单中: {}",
        plane.is_position_in_blacklist(2, 3)
    );
    println!(
        "  - (3,4) 是否在黑名单中: {}",
        plane.is_position_in_blacklist(3, 4)
    );

    // 从数据管理器加载权重字典
    println!("\n从数据管理器加载权重字典:");
    match BalancedRandDataManager::get_weights_by_plane_range(&[3, 4], "balanced_rand_data.json") {
        Ok(weights) => {
            for ((row, col), weight) in &weights {
                println!("  - ({}, {}): {:.3}", row, col, weight);
            }
        }
        Err(e) => eprintln!("加载权重字典失败: {}", e),
    }
}
