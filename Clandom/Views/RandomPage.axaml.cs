using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml;
using Clandom.Models.BalancedRandom;

namespace Clandom.Views;

public partial class RandomPage : UserControl
{   
    public RandomPage()
    {
        InitializeComponent();
    }

    private void Run_OnClick(object? sender, RoutedEventArgs e)
    {
        if (RMinId.Value >= RMaxId.Value)
        {
            return;
        }
        if (IsId.IsSelected)
        {
            var randId = new BalancedRand((int)RMinId.Value, (int)RMaxId.Value, minPoolSize: 3, maxGapThreshold: 3);
            Result.Text = randId.Draw().ToString();
        }
        else
        {
            var randId2D = new BalancedRand2D((int)RRaw.Value, (int)RCol.Value, minPoolSize: 3, maxGapThreshold: 3);
            var pos = randId2D.DrawPosition();
            Result.Text = $"行:{pos.row} 列:{pos.col}";
        }
    }
}