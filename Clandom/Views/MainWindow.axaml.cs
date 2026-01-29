using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using FluentAvalonia.UI.Controls;

namespace Clandom.Views;

public partial class MainWindow : Window
{
    public MainWindow()
    {
        InitializeComponent();
        var page = Activator.CreateInstance(Type.GetType("Clandom.Views.RandomPage") ?? throw new InvalidOperationException());
        NavigationView.Content = page;
    }

    private void NavigationView_OnSelectionChanged(object? sender, NavigationViewSelectionChangedEventArgs e)
    {
        if (e.IsSettingsSelected)
        {
            
        }
        else if (e.SelectedItem is NavigationViewItem item)
        {
            var prePage = $"Clandom.Views.{item.Tag}";
            var page = Activator.CreateInstance(Type.GetType(prePage) ?? throw new InvalidOperationException());
            (sender as NavigationView).Content = page;
        }
    }
}