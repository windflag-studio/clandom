using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.IO;

namespace Clandom.Models.BalancedRandom
{
    /// <summary>
    /// 平衡随机抽取数据存储结构
    /// </summary>
    public class BalancedRandData
    {
        public required string Id { get; set; }
        public DateTime LastUpdated { get; set; }
        public required Dictionary<int, int> DrawCounts { get; set; }
        public required Dictionary<int, int> LastDrawRound { get; set; }
        public int CurrentRound { get; set; }
        public int TotalDraws { get; set; }
        public required Dictionary<int, double> CurrentProbabilities { get; set; }
        
        // 配置参数
        public int MinPoolSize { get; set; }
        public int MaxGapThreshold { get; set; }
        public double ColdStartBoost { get; set; }
        public double DecayFactor { get; set; }
        
        // 用于类型识别的字段
        public required string Type { get; set; }
        
        // 用于2D类型的额外参数
        public int Rows { get; set; }
        public int Cols { get; set; }
        
        // 用于列表类型的参数
        public List<int> Numbers { get; set; }
        
        // 用于范围类型的参数
        public int NumberRangeStart { get; set; }
        public int NumberRangeEnd { get; set; }
    }
    
    /// <summary>
    /// 平衡随机抽取数据管理器
    /// </summary>
    public static class BalancedRandDataManager
    {
        private static readonly JsonSerializerOptions JsonOptions = new JsonSerializerOptions
        {
            WriteIndented = true,
            Converters = { new JsonStringEnumConverter() }
        };
        
        /// <summary>
        /// 加载所有保存的数据
        /// </summary>
        public static Dictionary<string, BalancedRandData> LoadAllData(string filePath = "balanced_rand_data.json")
        {
            try
            {
                if (File.Exists(filePath))
                {
                    string json = File.ReadAllText(filePath);
                    return JsonSerializer.Deserialize<Dictionary<string, BalancedRandData>>(json, JsonOptions) 
                        ?? new Dictionary<string, BalancedRandData>();
                }
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"加载数据失败: {ex.Message}");
            }
            
            return new Dictionary<string, BalancedRandData>();
        }
        
        /// <summary>
        /// 保存所有数据
        /// </summary>
        public static void SaveAllData(Dictionary<string, BalancedRandData> allData, 
                                      string filePath = "balanced_rand_data.json")
        {
            try
            {
                string json = JsonSerializer.Serialize(allData, JsonOptions);
                File.WriteAllText(filePath, json);
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"保存数据失败: {ex.Message}");
            }
        }
        
        /// <summary>
        /// 根据参数生成唯一ID
        /// </summary>
        public static string GenerateId(string type, params object[] parameters)
        {
            string paramString = string.Join("_", parameters.Select(p => p?.ToString() ?? "null"));
            return $"{type}_{paramString}";
        }
        
        /// <summary>
        /// 查找匹配的数据
        /// </summary>
        public static BalancedRandData FindMatchingData(Dictionary<string, BalancedRandData> allData, string type, params object[] parameters)
        {
            string targetId = GenerateId(type, parameters);
            
            // 精确匹配
            if (allData.ContainsKey(targetId))
            {
                return allData[targetId];
            }
            
            // 模糊匹配：查找相同类型和相似参数的数据
            foreach (var data in allData.Values)
            {
                if (data.Type == type)
                {
                    // 根据类型进行不同的匹配逻辑
                    if (type == "BalancedRand2D")
                    {
                        // 对于2D类型，检查行列是否匹配
                        // 这里可以根据需要实现更复杂的匹配逻辑
                    }
                }
            }
            
            return null;
        }
    }

    /// <summary>
    /// 平衡随机抽取类，提供智能动态权重算法和平均值差值保护机制
    /// </summary>
    public class BalancedRand
    {
        // 内部数据结构
        private Dictionary<int, int> _drawCounts;  // 学号 -> 抽取次数
        private Dictionary<int, int> _lastDrawRound;  // 学号 -> 最后被抽中的轮次
        private List<int> _allNumbers;  // 所有学号
        private List<int>? _candidatePool;  // 当前候选池
        private Random _random;
        
        // 配置参数
        private int _currentRound;  // 当前抽取轮次
        private int _minPoolSize;  // 最小候选池大小
        private int _maxGapThreshold;  // 最大差距阈值
        private double _coldStartBoost;  // 冷启动提升系数
        private double _decayFactor;  // 权重衰减因子
        
        // 统计信息
        private int _totalDraws;
        private Dictionary<int, double> _currentProbabilities;
        
        // 数据标识和类型
        private string _dataId;
        private string _type;
        
        // 构造函数参数
        private int _numberRangeStart;
        private int _numberRangeEnd;
        private List<int> _numbersList;

        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="numberRangeStart">学号起始值</param>
        /// <param name="numberRangeEnd">学号结束值</param>
        /// <param name="minPoolSize">最小候选池大小（默认3）</param>
        /// <param name="maxGapThreshold">最大抽取次数差距阈值（默认5）</param>
        /// <param name="coldStartBoost">冷启动提升系数（默认2.0）</param>
        /// <param name="decayFactor">权重衰减因子（默认0.7）</param>
        /// <param name="loadData">是否从文件加载历史数据（默认true）</param>
        public BalancedRand(int numberRangeStart, int numberRangeEnd, 
                           int minPoolSize = 3, int maxGapThreshold = 5,
                           double coldStartBoost = 2.0, double decayFactor = 0.7,
                           bool loadData = true)
        {
            if (numberRangeStart > numberRangeEnd)
                throw new ArgumentException("起始值不能大于结束值");
            
            if (minPoolSize < 1)
                throw new ArgumentException("最小候选池大小必须大于0");
                
            _allNumbers = Enumerable.Range(numberRangeStart, numberRangeEnd - numberRangeStart + 1).ToList();
            _drawCounts = _allNumbers.ToDictionary(n => n, _ => 0);
            _lastDrawRound = _allNumbers.ToDictionary(n => n, _ => -1); // -1表示从未被抽中
            _random = new Random(Guid.NewGuid().GetHashCode());
            _currentRound = 0;
            _minPoolSize = minPoolSize;
            _maxGapThreshold = maxGapThreshold;
            _coldStartBoost = coldStartBoost;
            _decayFactor = decayFactor;
            _totalDraws = 0;
            _currentProbabilities = new Dictionary<int, double>();
            
            // 保存构造函数参数
            _numberRangeStart = numberRangeStart;
            _numberRangeEnd = numberRangeEnd;
            _type = "BalancedRand_Range";
            
            // 生成数据ID
            _dataId = BalancedRandDataManager.GenerateId(_type, 
                numberRangeStart, numberRangeEnd, minPoolSize, maxGapThreshold, coldStartBoost, decayFactor);
            
            // 初始化候选池
            UpdateCandidatePool();
            
            // 加载历史数据
            if (loadData)
            {
                LoadData();
            }
        }

        /// <summary>
        /// 构造函数（通过列表指定学号）
        /// </summary>
        /// <param name="numbers">学号列表</param>
        /// <param name="minPoolSize">最小候选池大小</param>
        /// <param name="maxGapThreshold">最大抽取次数差距阈值</param>
        /// <param name="coldStartBoost">冷启动提升系数</param>
        /// <param name="decayFactor">权重衰减因子</param>
        /// <param name="loadData">是否从文件加载历史数据（默认true）</param>
        public BalancedRand(IEnumerable<int> numbers,
                           int minPoolSize = 3, int maxGapThreshold = 5,
                           double coldStartBoost = 2.0, double decayFactor = 0.7,
                           bool loadData = true)
        {
            var enumerable = numbers as int[] ?? numbers.ToArray();
            if (numbers == null || !enumerable.Any())
                throw new ArgumentException("学号列表不能为空");
                
            _allNumbers = enumerable.Distinct().ToList();
            _drawCounts = _allNumbers.ToDictionary(n => n, _ => 0);
            _lastDrawRound = _allNumbers.ToDictionary(n => n, _ => -1);
            _random = new Random(Guid.NewGuid().GetHashCode());
            _currentRound = 0;
            _minPoolSize = minPoolSize;
            _maxGapThreshold = maxGapThreshold;
            _coldStartBoost = coldStartBoost;
            _decayFactor = decayFactor;
            _totalDraws = 0;
            _currentProbabilities = new Dictionary<int, double>();
            
            // 保存构造函数参数
            _numbersList = new List<int>(_allNumbers);
            _type = "BalancedRand_List";
            
            // 生成数据ID
            _dataId = BalancedRandDataManager.GenerateId(_type, 
                string.Join(",", _allNumbers.OrderBy(n => n).Take(10)), // 取前10个学号作为标识
                minPoolSize, maxGapThreshold, coldStartBoost, decayFactor);
            
            UpdateCandidatePool();
            
            // 加载历史数据
            if (loadData)
            {
                LoadData();
            }
        }

        /// <summary>
        /// 从文件加载数据
        /// </summary>
        public virtual void LoadData(string filePath = "balanced_rand_data.json")
        {
            try
            {
                var allData = BalancedRandDataManager.LoadAllData(filePath);
                if (allData.TryGetValue(_dataId, out var savedData))
                {
                    ApplySavedData(savedData);
                    Debug.WriteLine($"已加载数据: {_dataId}");
                }
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"加载数据失败: {ex.Message}");
            }
        }

        /// <summary>
        /// 应用保存的数据
        /// </summary>
        protected virtual void ApplySavedData(BalancedRandData savedData)
        {
            // 只加载当前范围内的数据
            foreach (var kvp in savedData.DrawCounts)
            {
                if (_drawCounts.ContainsKey(kvp.Key))
                {
                    _drawCounts[kvp.Key] = kvp.Value;
                }
            }
            
            foreach (var kvp in savedData.LastDrawRound)
            {
                if (_lastDrawRound.ContainsKey(kvp.Key))
                {
                    _lastDrawRound[kvp.Key] = kvp.Value;
                }
            }
            
            _currentRound = savedData.CurrentRound;
            _totalDraws = savedData.TotalDraws;
            
            foreach (var kvp in savedData.CurrentProbabilities)
            {
                if (_currentProbabilities.ContainsKey(kvp.Key))
                {
                    _currentProbabilities[kvp.Key] = kvp.Value;
                }
            }
            
            // 更新配置参数（如果不同）
            _minPoolSize = savedData.MinPoolSize;
            _maxGapThreshold = savedData.MaxGapThreshold;
            _coldStartBoost = savedData.ColdStartBoost;
            _decayFactor = savedData.DecayFactor;
            
            // 更新候选池
            UpdateCandidatePool();
        }

        /// <summary>
        /// 保存数据到文件
        /// </summary>
        public virtual void SaveData(string filePath = "balanced_rand_data.json")
        {
            try
            {
                var allData = BalancedRandDataManager.LoadAllData(filePath);
                
                var data = new BalancedRandData
                {
                    Id = _dataId,
                    LastUpdated = DateTime.Now,
                    DrawCounts = new Dictionary<int, int>(_drawCounts),
                    LastDrawRound = new Dictionary<int, int>(_lastDrawRound),
                    CurrentRound = _currentRound,
                    TotalDraws = _totalDraws,
                    CurrentProbabilities = new Dictionary<int, double>(_currentProbabilities),
                    MinPoolSize = _minPoolSize,
                    MaxGapThreshold = _maxGapThreshold,
                    ColdStartBoost = _coldStartBoost,
                    DecayFactor = _decayFactor,
                    Type = _type
                };
                
                // 根据类型添加额外参数
                if (_type == "BalancedRand_Range")
                {
                    data.NumberRangeStart = _numberRangeStart;
                    data.NumberRangeEnd = _numberRangeEnd;
                }
                else if (_type == "BalancedRand_List" && _numbersList != null)
                {
                    data.Numbers = new List<int>(_numbersList);
                }
                
                allData[_dataId] = data;
                BalancedRandDataManager.SaveAllData(allData, filePath);
                
                Debug.WriteLine($"已保存数据: {_dataId}");
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"保存数据失败: {ex.Message}");
            }
        }

        /// <summary>
        /// 获取数据ID
        /// </summary>
        public string GetDataId() => _dataId;

        /// <summary>
        /// 获取类型
        /// </summary>
        public string GetTypeName() => _type;

        /// <summary>
        /// 获取最后抽取轮次字典（受保护访问器）
        /// </summary>
        protected Dictionary<int, int> GetLastDrawRoundInternal() => new Dictionary<int, int>(_lastDrawRound);

        /// <summary>
        /// 获取当前轮次（受保护访问器）
        /// </summary>
        protected int GetCurrentRoundInternal() => _currentRound;

        /// <summary>
        /// 获取总抽取次数（受保护访问器）
        /// </summary>
        protected int GetTotalDrawsInternal() => _totalDraws;

        /// <summary>
        /// 获取最小候选池大小（受保护访问器）
        /// </summary>
        protected int GetMinPoolSizeInternal() => _minPoolSize;

        /// <summary>
        /// 获取最大差距阈值（受保护访问器）
        /// </summary>
        protected int GetMaxGapThresholdInternal() => _maxGapThreshold;

        /// <summary>
        /// 获取冷启动提升系数（受保护访问器）
        /// </summary>
        protected double GetColdStartBoostInternal() => _coldStartBoost;

        /// <summary>
        /// 获取衰减因子（受保护访问器）
        /// </summary>
        protected double GetDecayFactorInternal() => _decayFactor;

        /// <summary>
        /// 抽取一个学号
        /// </summary>
        /// <param name="autoSave">是否自动保存数据（默认true）</param>
        /// <returns>抽取到的学号</returns>
        public int Draw(bool autoSave = true)
        {
            if (_candidatePool != null && _candidatePool.Count == 0)
            {
                // 如果候选池为空，重置所有抽取次数
                ResetDrawCounts();
            }

            _currentRound++;
            
            // 计算每个候选者的权重
            var weights = CalculateWeights();
            
            // 根据权重进行随机抽取
            int selectedNumber = WeightedRandomSelect(weights);
            
            // 更新抽取记录
            _drawCounts[selectedNumber]++;
            _lastDrawRound[selectedNumber] = _currentRound;
            _totalDraws++;
            
            // 更新候选池和概率
            UpdateCandidatePool();
            UpdateProbabilities();
            
            // 自动保存数据
            if (autoSave)
            {
                SaveData();
            }
            
            return selectedNumber;
        }

        /// <summary>
        /// 批量抽取多个学号
        /// </summary>
        /// <param name="count">抽取数量</param>
        /// <param name="autoSave">是否自动保存数据（默认true）</param>
        /// <returns>抽取到的学号列表</returns>
        public List<int> DrawMultiple(int count, bool autoSave = true)
        {
            if (count <= 0) 
                throw new ArgumentException("抽取数量必须大于0");
            if (_candidatePool != null && count > _candidatePool.Count)
                throw new ArgumentException($"抽取数量不能超过候选池大小({_candidatePool.Count})");
                
            List<int> results = new List<int>();
            
            for (int i = 0; i < count; i++)
            {
                // 每次抽取后候选池会更新，所以需要重新计算
                // 只在最后一次抽取后保存
                bool save = (i == count - 1) && autoSave;
                results.Add(Draw(save));
            }
            
            return results;
        }

        /// <summary>
        /// 获取当前抽取统计信息
        /// </summary>
        /// <returns>学号->抽取次数字典</returns>
        public Dictionary<int, int> GetStatistics()
        {
            return new Dictionary<int, int>(_drawCounts);
        }

        /// <summary>
        /// 获取当前每个学号的抽取概率
        /// </summary>
        /// <returns>学号->概率字典</returns>
        public Dictionary<int, double> GetProbabilities()
        {
            return new Dictionary<int, double>(_currentProbabilities);
        }

        /// <summary>
        /// 重置所有抽取次数（用于重新开始或手动平衡）
        /// </summary>
        public void ResetDrawCounts()
        {
            foreach (var number in _allNumbers)
            {
                _drawCounts[number] = 0;
                _lastDrawRound[number] = -1;
            }
            _totalDraws = 0;
            _currentRound = 0;
            UpdateCandidatePool();
        }

        /// <summary>
        /// 获取当前候选池
        /// </summary>
        /// <returns>候选池学号列表</returns>
        public List<int> GetCandidatePool()
        {
            Debug.Assert(_candidatePool != null, nameof(_candidatePool) + " != null");
            return new List<int>(_candidatePool);
        }

        /// <summary>
        /// 获取平均抽取次数
        /// </summary>
        /// <returns>平均抽取次数</returns>
        public double GetAverageDrawCount()
        {
            return _allNumbers.Count > 0 ? (double)_totalDraws / _allNumbers.Count : 0;
        }

        /// <summary>
        /// 获取最大抽取次数差距
        /// </summary>
        /// <returns>最大差距</returns>
        public int GetMaxDrawCountGap()
        {
            if (_drawCounts.Count == 0) return 0;
            int max = _drawCounts.Values.Max();
            int min = _drawCounts.Values.Min();
            return max - min;
        }

        /// <summary>
        /// 更新配置参数
        /// </summary>
        public void UpdateParameters(int? minPoolSize = null, int? maxGapThreshold = null,
                                   double? coldStartBoost = null, double? decayFactor = null)
        {
            if (minPoolSize.HasValue && minPoolSize.Value > 0)
                _minPoolSize = minPoolSize.Value;
                
            if (maxGapThreshold.HasValue && maxGapThreshold.Value >= 0)
                _maxGapThreshold = maxGapThreshold.Value;
                
            if (coldStartBoost.HasValue && coldStartBoost.Value >= 1.0)
                _coldStartBoost = coldStartBoost.Value;
                
            if (decayFactor.HasValue && decayFactor.Value > 0 && decayFactor.Value <= 1.0)
                _decayFactor = decayFactor.Value;
                
            UpdateCandidatePool();
        }

        #region 私有方法

        /// <summary>
        /// 更新候选池
        /// </summary>
        private void UpdateCandidatePool()
        {
            // 计算平均抽取次数
            double average = GetAverageDrawCount();
            
            // 第一步：平均值过滤 - 只选择抽取次数≤平均值的成员
            var candidates = _allNumbers
                .Where(n => _drawCounts[n] <= Math.Ceiling(average)) // 向上取整，增加容错
                .ToList();
            
            // 第二步：最大差距保护
            if (GetMaxDrawCountGap() > _maxGapThreshold)
            {
                // 排除极值并重新计算
                int maxCount = _drawCounts.Values.Max();
                int minCount = _drawCounts.Values.Min();
                
                // 排除抽取次数最多和最少的成员
                candidates = candidates
                    .Where(n => _drawCounts[n] != maxCount && _drawCounts[n] != minCount)
                    .ToList();
                
                // 重新计算排除极值后的平均值
                if (candidates.Any())
                {
                    double newAverage = candidates.Average(n => _drawCounts[n]);
                    candidates = candidates
                        .Where(n => _drawCounts[n] <= Math.Ceiling(newAverage))
                        .ToList();
                }
            }
            
            // 第三步：候选池大小保障
            if (candidates.Count < _minPoolSize)
            {
                // 如果候选池太小，添加一些抽取次数较低的成员
                var allSorted = _allNumbers
                    .OrderBy(n => _drawCounts[n])
                    .ThenBy(n => _lastDrawRound[n]) // 长期未抽中的优先
                    .ToList();
                    
                int needed = _minPoolSize - candidates.Count;
                foreach (var number in allSorted)
                {
                    if (!candidates.Contains(number) && needed > 0)
                    {
                        candidates.Add(number);
                        needed--;
                    }
                }
            }
            
            _candidatePool = candidates;
        }

        /// <summary>
        /// 计算权重
        /// </summary>
        private Dictionary<int, double> CalculateWeights()
        {
            var weights = new Dictionary<int, double>();

            if (_candidatePool != null)
                foreach (var number in _candidatePool)
                {
                    double weight = 1.0;

                    // 1. 基础权重：避免重复抽取
                    weight *= Math.Pow(_decayFactor, _drawCounts[number]);

                    // 2. 冷启动保护：长期未被抽中的成员权重提升
                    if (_lastDrawRound[number] < 0) // 从未被抽中
                    {
                        weight *= _coldStartBoost;
                    }
                    else
                    {
                        int roundsSinceLastDraw = _currentRound - _lastDrawRound[number];
                        if (roundsSinceLastDraw > _allNumbers.Count / 2) // 超过一半轮次未抽中
                        {
                            weight *= (1.0 + Math.Log(roundsSinceLastDraw + 1) / 10.0);
                        }
                    }

                    // 3. 抽取次数倒数权重（抽取越多，权重越低）
                    weight *= 1.0 / (_drawCounts[number] + 1);

                    weights[number] = Math.Max(weight, 0.01); // 保证最小权重
                }

            return weights;
        }

        /// <summary>
        /// 根据权重进行随机选择
        /// </summary>
        private int WeightedRandomSelect(Dictionary<int, double> weights)
        {
            if (!weights.Any())
                throw new InvalidOperationException("权重字典为空");
                
            // 计算总权重
            double totalWeight = weights.Values.Sum();
            
            // 生成随机数
            double randomValue = _random.NextDouble() * totalWeight;
            
            // 根据权重选择
            double cumulative = 0;
            foreach (var kvp in weights)
            {
                cumulative += kvp.Value;
                if (randomValue <= cumulative)
                {
                    return kvp.Key;
                }
            }
            
            // 如果由于浮点精度问题未选择，返回最后一个
            return weights.Keys.Last();
        }

        /// <summary>
        /// 更新概率信息
        /// </summary>
        private void UpdateProbabilities()
        {
            _currentProbabilities.Clear();
            
            if (_candidatePool != null && _candidatePool.Count == 0) return;
            
            var weights = CalculateWeights();
            double totalWeight = weights.Values.Sum();
            
            foreach (var kvp in weights)
            {
                _currentProbabilities[kvp.Key] = kvp.Value / totalWeight;
            }
            
            // 为不在候选池中的成员设置概率为0
            foreach (var number in _allNumbers.Where(n => _candidatePool != null && !_candidatePool.Contains(n)))
            {
                _currentProbabilities[number] = 0;
            }
        }

        #endregion
    }

    /// <summary>
    /// 扩展类：支持按行列抽取（模拟二维数组）
    /// </summary>
    public class BalancedRand2D : BalancedRand
    {
        private int _rows;
        private int _cols;
        private string _dataId2D;
        
        /// <summary>
        /// 构造函数
        /// </summary>
        /// <param name="rows">行数</param>
        /// <param name="cols">列数</param>
        /// <param name="minPoolSize">最小候选池大小</param>
        /// <param name="maxGapThreshold">最大抽取次数差距阈值</param>
        /// <param name="coldStartBoost">冷启动提升系数</param>
        /// <param name="decayFactor">权重衰减因子</param>
        /// <param name="loadData">是否从文件加载历史数据（默认true）</param>
        public BalancedRand2D(int rows, int cols, int minPoolSize = 3, 
                            int maxGapThreshold = 5, double coldStartBoost = 2.0, 
                            double decayFactor = 0.7, bool loadData = true) 
            : base(0, rows * cols - 1, minPoolSize, maxGapThreshold, coldStartBoost, decayFactor, false)
        {
            _rows = rows;
            _cols = cols;
            
            // 生成2D专用的数据ID
            _dataId2D = BalancedRandDataManager.GenerateId("BalancedRand2D", 
                rows, cols, minPoolSize, maxGapThreshold, coldStartBoost, decayFactor);
            
            // 加载历史数据
            if (loadData)
            {
                LoadData();
            }
        }
        
        /// <summary>
        /// 从文件加载数据（重写以使用2D专用ID）
        /// </summary>
        public override void LoadData(string filePath = "balanced_rand_data.json")
        {
            try
            {
                var allData = BalancedRandDataManager.LoadAllData(filePath);
                
                // 优先使用2D专用ID，如果没有则尝试使用基类ID
                if (allData.TryGetValue(_dataId2D, out var savedData) || 
                    allData.TryGetValue(GetDataId(), out savedData))
                {
                    ApplySavedData(savedData);
                    Debug.WriteLine($"已加载2D数据: {_dataId2D}");
                }
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"加载2D数据失败: {ex.Message}");
            }
        }
        
        /// <summary>
        /// 应用保存的数据（重写以处理2D数据）
        /// </summary>
        protected override void ApplySavedData(BalancedRandData savedData)
        {
            // 调用基类方法
            base.ApplySavedData(savedData);
            
            // 2D特有的处理逻辑
            if (savedData.Rows > 0 && savedData.Cols > 0)
            {
                // 如果保存的数据中有行列信息，可以在这里处理
            }
        }
        
        /// <summary>
        /// 保存数据到文件（重写以使用2D专用ID）
        /// </summary>
        public override void SaveData(string filePath = "balanced_rand_data.json")
        {
            try
            {
                var allData = BalancedRandDataManager.LoadAllData(filePath);
                
                var data = new BalancedRandData
                {
                    Id = _dataId2D,
                    LastUpdated = DateTime.Now,
                    DrawCounts = GetStatistics(),
                    LastDrawRound = GetLastDrawRoundInternal(),
                    CurrentRound = GetCurrentRoundInternal(),
                    TotalDraws = GetTotalDrawsInternal(),
                    CurrentProbabilities = GetProbabilities(),
                    MinPoolSize = GetMinPoolSizeInternal(),
                    MaxGapThreshold = GetMaxGapThresholdInternal(),
                    ColdStartBoost = GetColdStartBoostInternal(),
                    DecayFactor = GetDecayFactorInternal(),
                    Type = "BalancedRand2D",
                    Rows = _rows,
                    Cols = _cols
                };
                
                allData[_dataId2D] = data;
                BalancedRandDataManager.SaveAllData(allData, filePath);
                
                Debug.WriteLine($"已保存2D数据: {_dataId2D}");
            }
            catch (Exception ex)
            {
                Debug.WriteLine($"保存2D数据失败: {ex.Message}");
            }
        }
        
        /// <summary>
        /// 获取数据ID（2D专用）
        /// </summary>
        public new string GetDataId() => _dataId2D;
        
        /// <summary>
        /// 抽取一个位置（返回行列，1-based索引）
        /// </summary>
        /// <param name="autoSave">是否自动保存数据（默认true）</param>
        /// <returns>(行, 列)，行和列从1开始</returns>
        public (int row, int col) DrawPosition(bool autoSave = true)
        {
            int number = Draw(autoSave);
            // 将1-based学号转换为1-based行列
            int zeroBasedNumber = number - 1;
            return (zeroBasedNumber / _cols + 1, zeroBasedNumber % _cols + 1);
        }
        
        /// <summary>
        /// 批量抽取多个位置（1-based索引）
        /// </summary>
        /// <param name="count">抽取数量</param>
        /// <param name="autoSave">是否自动保存数据（默认true）</param>
        public List<(int row, int col)> DrawMultiplePositions(int count, bool autoSave = true)
        {
            var numbers = DrawMultiple(count, autoSave);
            return numbers.Select(n => 
            {
                int zeroBasedNumber = n - 1;
                return (zeroBasedNumber / _cols + 1, zeroBasedNumber % _cols + 1);
            }).ToList();
        }
        
        /// <summary>
        /// 获取位置统计信息（1-based索引）
        /// </summary>
        public Dictionary<(int row, int col), int> GetPositionStatistics()
        {
            var stats = GetStatistics();
            return stats.ToDictionary(
                kv => 
                {
                    int zeroBasedNumber = kv.Key - 1;
                    return (zeroBasedNumber / _cols + 1, zeroBasedNumber % _cols + 1);
                },
                kv => kv.Value
            );
        }
        
        /// <summary>
        /// 获取位置概率信息（1-based索引）
        /// </summary>
        public Dictionary<(int row, int col), double> GetPositionProbabilities()
        {
            var probs = GetProbabilities();
            return probs.ToDictionary(
                kv => 
                {
                    int zeroBasedNumber = kv.Key - 1;
                    return (zeroBasedNumber / _cols + 1, zeroBasedNumber % _cols + 1);
                },
                kv => kv.Value
            );
        }
    }
}