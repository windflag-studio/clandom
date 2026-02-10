import { Stack, ToggleButton, ToggleButtonGroup } from '@mui/material';
import Grid3x3RoundedIcon from '@mui/icons-material/Grid3x3Rounded';
import FaceRoundedIcon from '@mui/icons-material/FaceRounded';

export default function drawPage() {
  return (
    <>
      <Stack spacing={2} direction={'column'}>
        <Stack direction={'row'}>
          <ToggleButtonGroup>
            <ToggleButton value={"idMode"} aria-label='idMode'>
              <FaceRoundedIcon />
            </ToggleButton>
            <ToggleButton value={"planeMode"} aria-label='planeMode'>
              <Grid3x3RoundedIcon />
            </ToggleButton>
          </ToggleButtonGroup>
        </Stack>
      </Stack>
    </>
  );
}