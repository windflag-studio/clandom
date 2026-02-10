import { Stack, ToggleButton, ToggleButtonGroup, Button } from '@mui/material';
import Grid3x3RoundedIcon from '@mui/icons-material/Grid3x3Rounded';
import FaceRoundedIcon from '@mui/icons-material/FaceRounded';
import * as React from 'react';
import { styled } from '@mui/material/styles';
import Paper from '@mui/material/Paper';
import NumberSpinner from '../Components/NumberSpinner';
import { invoke } from '@tauri-apps/api/core';

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

export default function drawPage() {
  const [alignment, setAlignment] = React.useState<string | null>('left');

  const handleAlignment = (
    _: React.MouseEvent<HTMLElement>,
    newAlignment: string | null,
  ) => {
    setAlignment(newAlignment);
  };

  return (
    <>
      <Stack spacing={2} direction={'column'}
        sx={{
            justifyContent: 'center'
          }}>
        <Stack direction={'row'} spacing={2}
          sx={{
            justifyContent: 'center',
            alignItems: 'center'
          }}>
          <Item>模式:</Item>
          <ToggleButtonGroup exclusive aria-label='mode' value={alignment} onChange={handleAlignment}>
            <ToggleButton value={"idMode"} aria-label='idMode'>
              <FaceRoundedIcon />
            </ToggleButton>
            <ToggleButton value={"planeMode"} aria-label='planeMode'>
              <Grid3x3RoundedIcon />
            </ToggleButton>
          </ToggleButtonGroup>
          {alignment === 'idMode'?
          (<IdRangeComponent />):
          (<PlaneRangeComponent />)
          }
          <Button variant="contained" onClick={
            () => {
              
            }
          }>Contained</Button>
        </Stack>
      </Stack>
    </>
  );
}

function IdRangeComponent() {
  const [minId, setMinId] = React.useState<number>(1);
  const [maxId, setMaxId] = React.useState<number>(50);

  const handleMinId = (value: number | null, _: any) => {
    if (value !== null) {
      setMinId(value);
      if(maxId <= value) {
        setMaxId(value + 1);
      }
    }
  }
  const handleMaxId = (value: number | null, _: any) => {
    if (value !== null) {
      setMaxId(value);
    }
  }
  return (
    <>
      <NumberSpinner
        name="minId"
        label="最小学号"
        min={1}
        max={10000}
        value={minId}
        onValueChange={handleMinId}
      />
      <NumberSpinner
        name="maxId"
        label="最大学号"
        min={minId + 1}
        max={10001}
        value={maxId}
        onValueChange={handleMaxId}
      />
    </>
  )
}

function PlaneRangeComponent() {
  const [rowNum, setRowNum] = React.useState<number>(6);
  const [colNum, setColNum] = React.useState<number>(8);

  const handleRowNum = (value: number | null, _: any) => {
    if (value !== null) {
      setRowNum(value);
      if(colNum <= value) {
        setColNum(value + 1);
      }
    }
  }
  const handleColNum = (value: number | null, _: any) => {
    if (value !== null) {
      setColNum(value);
    }
  }
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