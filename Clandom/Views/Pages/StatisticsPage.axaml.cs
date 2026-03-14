using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml;
using Clandom.Models.BalancedRandom;
using Clandom.ViewModels;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Clandom.Views.Pages;

public partial class StatisticsPage : UserControl
{
    private List<List<int>> _idData;
    private List<List<int>> _planeData;
    public StatisticsPage()
    {
        InitializeComponent();
    }

    private void Control_OnLoaded(object? sender, RoutedEventArgs e)
    {
        _idData = BalancedRandDataManager.GetAllIdData();
        _planeData = BalancedRandDataManager.GetAllPlaneData();
        foreach (var dataId in _idData)
        {
            IdStatisticsComboBox.Items.Add($"从{dataId[0]}到{dataId[1]}");
        }
        IdStatisticsComboBox.SelectedIndex = 0;
        
        foreach (var dataTd in _planeData)
        {
            PlaneStatisticsComboBox.Items.Add($"{dataTd[0]}行{dataTd[1]}列");
        }
        PlaneStatisticsComboBox.SelectedIndex = 0;
    }

    private void IdStatisticsComboBox_OnSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (IsInitialized&&_idData.Count != 0)
        {
            StatisticsPageViewModel.IdCountsData = BalancedRandDataManager.GetDrawCountsByIdRange(_idData[IdStatisticsComboBox.SelectedIndex]).ToArray();
            StatisticsPageViewModel.IdWeightData = BalancedRandDataManager.GetWeightsByIdRange(_idData[IdStatisticsComboBox.SelectedIndex]).ToArray();
            (DataContext as StatisticsPageViewModel).RefreshIdSeries();
        }
    }

    private void TDStatisticsComboBox_OnSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (IsInitialized && _planeData != null && _planeData.Count > 0 && 
            PlaneStatisticsComboBox.SelectedIndex >= 0)
        {
            try
            {
                var countsDict = BalancedRandDataManager.GetDrawCountsByPlaneRange(
                    _planeData[PlaneStatisticsComboBox.SelectedIndex]);
                var weightsDict = BalancedRandDataManager.GetWeightsByPlaneRange(
                    _planeData[PlaneStatisticsComboBox.SelectedIndex]);
                
                // 将字典转换为有序数组
                var orderedCounts = new List<int>();
                var orderedWeights = new List<double>();
                var orderedLabels = new List<string>();
                
                // 按行列顺序排序
                var rows = _planeData[PlaneStatisticsComboBox.SelectedIndex][0];
                var cols = _planeData[PlaneStatisticsComboBox.SelectedIndex][1];
                
                for (int row = 0; row < rows; row++)
                {
                    for (int col = 0; col < cols; col++)
                    {
                        var key = new List<int> { col, row }; // 注意: GetPlaneConfigDrawCounts 返回的是 [col, row]
                        if (countsDict.TryGetValue(key, out var count))
                            orderedCounts.Add(count);
                        else
                            orderedCounts.Add(0);
                            
                        if (weightsDict.TryGetValue(key, out var weight))
                            orderedWeights.Add(weight);
                        else
                            orderedWeights.Add(0);
                            
                        orderedLabels.Add($"[{row+1},{col+1}]");
                    }
                }
                
                StatisticsPageViewModel.PlaneCountsData = orderedCounts.ToArray();
                StatisticsPageViewModel.PlaneWeightData = orderedWeights.ToArray();
                (DataContext as StatisticsPageViewModel).PlaneLabelData = orderedLabels.ToArray();
                (DataContext as StatisticsPageViewModel).RefreshPlaneSeries();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"加载平面统计数据失败: {ex.Message}");
            }
        }
    }
}