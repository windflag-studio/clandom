using CommunityToolkit.Mvvm.ComponentModel;

namespace Clandom.ViewModels;

partial class FloatViewModel : ViewModelBase
{
    [ObservableProperty] private int _width;
}