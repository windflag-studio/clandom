import { Stack, ToggleButton, ToggleButtonGroup, Button, Alert, Slider, Tooltip } from '@mui/material';
import type { SliderValueLabelProps } from '@mui/material/Slider';
import Grid3x3RoundedIcon from '@mui/icons-material/Grid3x3Rounded';
import FaceRoundedIcon from '@mui/icons-material/FaceRounded';
import * as React from 'react';
import { styled } from '@mui/material/styles';
import Paper from '@mui/material/Paper';
import NumberSpinner from '../Components/NumberSpinner';
import { invoke } from '@tauri-apps/api/core';
import ContainerAwareText from '../Components/ContainerAwareText';

const minDistance = 5;
const maxId = 190;

const idMarks = [
  {
    value: 1,
    label: '1',
  },
  {
    value: 50,
    label: '50',
  },
  {
    value: 100,
    label: '100',
  },
  {
    value: 190,
    label: '1000',
  },
];

const Item = styled(Paper)(({ theme }) => ({
  backgroundColor: '#fff',
  ...theme.typography.body2,
  padding: theme.spacing(1),
  textAlign: 'center',
  color: (theme.vars ?? theme).palette.text.secondary,
  flexGrow: 1,
  ...theme.applyStyles('dark', {
    backgroundColor: '#1A2027',
  }),
}));

export default function DrawPage() {
  // const [minId, setMinId] = React.useState<number>(1);
  // const [maxId, setMaxId] = React.useState<number>(50);
  const [rowNum, setRowNum] = React.useState<number>(6);
  const [colNum, setColNum] = React.useState<number>(8);
  const [alignment, setAlignment] = React.useState<string | null>('idMode');
  const [result, setResult] = React.useState<string>('等待抽取...');
  const [loading, setLoading] = React.useState<boolean>(false);
  const [error, setError] = React.useState<string | null>(null);
  const [idRange, setIdRange] = React.useState<number[]>([20, 37]);

  const handleIdRange = (_: Event, newValue: number[], activeThumb: number) => {
    if (newValue[1] - newValue[0] < minDistance) {
      if (activeThumb === 0) {
        const clamped = Math.min(newValue[0], maxId - minDistance);
        setIdRange([clamped, clamped + minDistance]);
      } else {
        const clamped = Math.max(newValue[1], minDistance);
        setIdRange([clamped - minDistance, clamped]);
      }
    } else {
      setIdRange(newValue);
    }
  };

  const handleRowNum = (value: number | null, _: any) => {
    if (value !== null) {
      setRowNum(value);
      if (colNum <= value) {
        setColNum(value + 1);
      }
    }
  };

  const handleColNum = (value: number | null, _: any) => {
    if (value !== null) {
      setColNum(value);
    }
  };

  const handleAlignment = (
    _: React.MouseEvent<HTMLElement>,
    newAlignment: string | null,
  ) => {
    setAlignment(newAlignment);
  };

  const handleDraw = async () => {
    setLoading(true);
    setError(null);
    //setResult('抽取中...');

    try {
      let res;
      if (alignment === 'idMode') {
        res = await invoke('draw_id', {
          minId: calculateIdValue(idRange[0]),
          maxId: calculateIdValue(idRange[1])
        });
      } else {
        res = await invoke('draw_plane', {
          rowNum,
          colNum
        });
      }
      setResult(`${res}`);
    } catch (err: any) {
      const errorMessage = err.toString();
      setError(`错误: ${errorMessage}`);
      setResult('抽取失败');

      // 检查是否是 Tauri 环境问题
      if (errorMessage.includes('undefined') || errorMessage.includes('invoke')) {
        console.error('Tauri invoke 不可用，请确保在 Tauri 环境中运行');
        console.error('当前运行环境:', typeof window !== 'undefined' ? '浏览器' : '未知');
        console.error('window.__TAURI__ 存在?', !!(window as any).__TAURI__);
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <Stack spacing={2}
      sx={{
        flex: 1,
        overflow: 'auto',
        width: '100%',
        height: '100%'
      }}>
      <Stack direction={'row'} spacing={2}
        sx={{
          justifyContent: 'center',
          alignItems: 'center'
        }}>
        <Item>模式:</Item>
        <ToggleButtonGroup size="small" exclusive aria-label='mode' value={alignment} onChange={handleAlignment}>
          <ToggleButton value={"idMode"} aria-label='idMode'>
            <FaceRoundedIcon />
          </ToggleButton>
          <ToggleButton value={"planeMode"} aria-label='planeMode'>
            <Grid3x3RoundedIcon />
          </ToggleButton>
        </ToggleButtonGroup>
        {alignment === 'idMode' ?
          (<IdRangeComponent idRange={idRange} handleIdRange={handleIdRange} />) :
          (<PlaneRangeComponent rowNum={rowNum} colNum={colNum} handleRowNum={handleRowNum} handleColNum={handleColNum} />)
        }
        <Button
          variant="contained"
          onClick={handleDraw}
          disabled={loading}
        >
          抽！
        </Button>
      </Stack>

      <div
        style={{
          flexGrow: 1,
          minWidth: 0
        }}>
        {error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}

        <Item sx={{
          flexGrow: 1,
          minHeight: 60,
          display: 'flex',
          fontWeight: 'bold',
          justifyContent: "center",
          alignItems: "center",
          height: '100%'
        }}>
          <ContainerAwareText minFontSize={15} maxFontSize={300} scaleRatio={0.6}>
            {result}
          </ContainerAwareText>
        </Item>
      </div>
    </Stack>
  );
}

function calculateIdValue(value: number) {
  if (1 <= value && value <= 100) {
    return value;
  }
  else if (value > 100) {
    return 100 + (value - 100) * 10;
  }
  else {
    return 1;
  }
}

function idLabelFormat(value: number) {
  return `${calculateIdValue(value)}`;
}

interface IdRangeComponentProps {
  idRange: number[];
  handleIdRange: (event: Event, newValue: number[], activeThumb: number) => void;
}
function IdValueLabelComponent(props: SliderValueLabelProps) {
  const { children, value } = props;
  let resultValue = calculateIdValue(Number(value));
  
  return (
    <Tooltip enterTouchDelay={0} placement="top" title={resultValue}>
      {children}
    </Tooltip>
  );
}
function IdRangeComponent({ idRange, handleIdRange }: IdRangeComponentProps) {
  return (
    <Slider
      min={1}
      max={maxId}
      getAriaLabel={() => '学号范围'}
      value={idRange}
      onChange={handleIdRange}
      valueLabelDisplay='auto'
      marks={idMarks}
      slots={{
        valueLabel: IdValueLabelComponent,
      }}
      getAriaValueText={idLabelFormat}
      disableSwap
    />
  )
}

interface PlaneRangeComponentProps {
  rowNum: number;
  colNum: number;
  handleRowNum: (value: number | null, _: any) => void;
  handleColNum: (value: number | null, _: any) => void;
}

function PlaneRangeComponent({ rowNum, colNum, handleRowNum, handleColNum }: PlaneRangeComponentProps) {
  return (
    <>
      <NumberSpinner
        name="rowNum"
        label="行"
        min={1}
        max={100}
        value={rowNum}
        onValueChange={handleRowNum}
      />
      <NumberSpinner
        name="colNum"
        label="列"
        min={1}
        max={100}
        value={colNum}
        onValueChange={handleColNum}
      />
    </>
  )
}