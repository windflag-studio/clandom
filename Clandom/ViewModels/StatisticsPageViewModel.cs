using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using LiveChartsCore;
using LiveChartsCore.SkiaSharpView;

namespace Clandom.ViewModels;

partial class StatisticsPageViewModel : ViewModelBase
{
    public static int[] IdCountsData;
    public static double[] IdWeightData;
    public static int[] PlaneCountsData;
    public static double[] PlaneWeightData;
    
    [ObservableProperty]
    private IList<string> _planeLabelData;
    
    // 拆分成四个独立的系列
    [ObservableProperty]
    private ISeries[] _idCountsSeries;
    
    [ObservableProperty]
    private ISeries[] _idWeightSeries;
    
    [ObservableProperty]
    private ISeries[] _planeCountsSeries;
    
    [ObservableProperty]
    private ISeries[] _planeWeightSeries;
    
    public Func<double, string> Labeler { get; set; } =
        value => value.ToString("N2");
    public int IdMax => Math.Max(IdCountsData.Length, IdWeightData.Length);
    public int PlaneMax => Math.Max(PlaneCountsData.Length, PlaneWeightData.Length);
    
    public StatisticsPageViewModel()
    {
        RefreshIdSeries();
        RefreshPlaneSeries();
    }
    
    public void RefreshIdSeries()
    {
        // ID抽取次数（柱状图）
        IdCountsSeries = new ISeries[]
        {
            new ColumnSeries<int>
            {
                Values = IdCountsData ?? new int[0],
                Name = "抽取次数"
            }
        };
        
        // ID权重（折线图）
        IdWeightSeries = new ISeries[]
        {
            new LineSeries<double>
            {
                Values = IdWeightData ?? new double[0],
                Name = "权重"
            }
        };
    }
    public void RefreshPlaneSeries()
    {
        // 平面抽取次数（柱状图）
        PlaneCountsSeries = new ISeries[]
        {
            new ColumnSeries<int>
            {
                Values = PlaneCountsData ?? new int[0],
                Name = "抽取次数"
            }
        };
        
        // 平面权重（折线图）
        PlaneWeightSeries = new ISeries[]
        {
            new LineSeries<double>
            {
                Values = PlaneWeightData ?? new double[0],
                Name = "权重"
            }
        };
    }
}