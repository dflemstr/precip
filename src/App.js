import React from 'react'
import Grid from '@material-ui/core/Grid'
import Plant from './Plant'
import LinearProgress from '@material-ui/core/LinearProgress'
import Error from '@material-ui/icons/Error'
import Typography from '@material-ui/core/Typography'
import Card from '@material-ui/core/Card'
import CardContent from '@material-ui/core/CardContent'

/* global fetch */

class App extends React.Component {
  constructor (props) {
    super(props)
    this.controller = 'AbortController' in window ? new window.AbortController() : null
    this.interval = null
    this.state = {error: null, data: null}
    this.mounted = false
  }

  componentDidMount () {
    this.mounted = true
    this._update()
    this.interval = setInterval(() => this._update(), 60 * 1000)
  }

  _update () {
    fetch('https://s3-eu-west-1.amazonaws.com/precip-stats/data.json', {
      headers: {
        'accept': 'application/json'
      },
      signal: this.controller ? this.controller.signal : null
    })
      .then(response => response.json())
      .catch(error => {
        if (this.mounted) {
          this.setState({...this.state, error})
        }
      })
      .then(data => {
        if (this.mounted) {
          this.setState({...this.state, data})
        }
      })
  }

  componentWillUnmount () {
    if (this.controller) {
      this.controller.abort()
    }

    if (this.interval !== null) {
      clearInterval(this.interval)
      this.interval = null
    }

    this.mounted = false
  }

  render () {
    if (this.state.error) {
      return (<main style={{maxWidth: '960px', margin: '24px auto'}}>
        <Card>
          <CardContent>
            <Typography gutterBottom variant='headline'>
              <Error style={{verticalAlign: 'text-top'}} /> Error
            </Typography>
            <Typography>{this.state.error.message}</Typography>
          </CardContent>
        </Card>
      </main>)
    } else if (this.state.data) {
      let items = this.state.data.modules.map(module => (<Grid key={module.id} item>
        <Plant title={module.name} historicalMoisture={module.historical_moisture} />
      </Grid>))
      return (<main style={{maxWidth: '960px', margin: '24px auto'}}>
        <Grid container spacing={24}>
          {items}
        </Grid>
      </main>)
    } else {
      return <LinearProgress />
    }
  }
}

export default App
