using System;
using System.Collections.Generic;
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
        if (IsInitialized&&_planeData.Count != 0)
        {
            StatisticsPageViewModel.PlaneCountsData = BalancedRandDataManager.GetDrawCountsByPlaneRange(_planeData[PlaneStatisticsComboBox.SelectedIndex]).ToArray();
            StatisticsPageViewModel.PlaneWeightData = BalancedRandDataManager.GetWeightsByPlaneRange(_planeData[PlaneStatisticsComboBox.SelectedIndex]).ToArray();
            (DataContext as StatisticsPageViewModel).RefreshPlaneSeries();
        }
    }
}