using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml;
using Clandom.Models.BalancedRandom;

namespace Clandom.Views.Pages;

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
            var randId = new BalancedRand((int)RMinId.Value, (int)RMaxId.Value);
            Result.Text = randId.Draw().ToString();
        }
        else
        {
            var randIdPlane = new BalancedRandPlane((int)RRaw.Value, (int)RCol.Value);
            var pos = randIdPlane.DrawPosition();
            Result.Text = $"行:{pos.row} 列:{pos.col}";
        }
    }
}