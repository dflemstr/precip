import React from 'react'
import Grid from '@material-ui/core/Grid'
import Plant from './Plant'

const App = () => {
  return (<main style={{maxWidth: '960px', margin: '24px auto'}}>
    <Grid container spacing={24}>
      <Grid item>
        <Plant title='Plant 1' />
      </Grid>
      <Grid item>
        <Plant title='Plant 2' />
      </Grid>
      <Grid item>
        <Plant title='Plant 3' />
      </Grid>
      <Grid item>
        <Plant title='Plant 4' />
      </Grid>
    </Grid>
  </main>)
}

export default App
