import React from 'react'
import Grid from '@material-ui/core/Grid'
import Plant from './Plant'
import LinearProgress from '@material-ui/core/LinearProgress'
import Error from '@material-ui/icons/Error'
import Typography from '@material-ui/core/Typography'
import Card from '@material-ui/core/Card'
import CardContent from '@material-ui/core/CardContent'
import { withStyles } from '@material-ui/core/styles'

const styles = theme => ({
  root: {
    flexGrow: 1,
    padding: theme.spacing.unit * 3
  },
  error: {
    verticalAlign: 'text-top'
  }
})

class App extends React.Component {
  constructor (props) {
    super(props)
    this.controller = typeof window.AbortController !== 'undefined' ? new window.AbortController() : null
    this.interval = null
    this.state = {error: null, data: null}
    this.mounted = false
  }

  async componentDidMount () {
    this.mounted = true
    this.interval = setInterval(() => this._update(), 60 * 1000)
    await this._update()
  }

  async _update () {
    try {
      const response = await window.fetch('https://s3-eu-west-1.amazonaws.com/precip-stats/data.json', {
        headers: {
          'accept': 'application/json'
        },
        signal: this.controller ? this.controller.signal : null
      })
      const data = await response.json()
      this.onDataReceived(data)
    } catch (error) {
      this.onDataError(error)
    }
  }

  onDataReceived (data) {
    if (this.mounted) {
      this.setState({...this.state, data, error: null})
    }
  }

  onDataError (error) {
    if (this.mounted) {
      this.setState({...this.state, error, data: null})
    }
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
    const {classes} = this.props
    if (this.state.error) {
      return (<main className={classes.root}>
        <Grid container justify='center' spacing={24}>
          <Grid item>
            <Card>
              <CardContent>
                <Typography gutterBottom variant='headline'>
                  <Error className={classes.error} /> Error
                </Typography>
                <Typography>{this.state.error.message}</Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </main>)
    } else if (this.state.data) {
      let items = this.state.data.modules.map(module => (<Grid key={module.id} item>
        <Plant title={module.name} module={module} />
      </Grid>))
      return (<main className={classes.root}>
        <Grid container spacing={24}>
          {items}
        </Grid>
      </main>)
    } else {
      return <LinearProgress />
    }
  }
}

export default withStyles(styles)(App)
