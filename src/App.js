import React from 'react'
import Grid from '@material-ui/core/Grid'
import Plant from './Plant'
import LinearProgress from '@material-ui/core/LinearProgress'
import Error from '@material-ui/icons/Error'
import Typography from '@material-ui/core/Typography'
import Card from '@material-ui/core/Card'
import CardContent from '@material-ui/core/CardContent'
import { withStyles } from '@material-ui/core/styles'
import backgroundImage from './background.jpg?sizes[]=300,sizes[]=600,sizes[]=1200,sizes[]=2000'
import Moment from 'react-moment'
import { sortBy } from 'lodash/collection'

const styles = theme => ({
  root: {
    flexGrow: 1,
    width: '100%',
    height: '100%'
  },
  background: {
    position: 'fixed',
    top: 0,
    left: 0,
    objectFit: 'cover',
    width: '100%',
    height: '100%',
    zIndex: '-200'
  },
  grid: {
    maxWidth: '960px',
    margin: '0 auto'
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
      data.modules = sortBy(data.modules, 'name')
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
    const background =
      <img className={classes.background} alt='' src={backgroundImage.src} srcSet={backgroundImage.srcSet} />

    if (this.state.error) {
      return (<main className={classes.root}>
        {background}
        <Grid className={classes.grid} container justify='center' spacing={32}>
          <Grid item xs={12}>
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
        {background}
        <Grid className={classes.grid} container spacing={32}>
          <Grid item xs={12}>
            <Card>
              <CardContent>
                <Typography gutterBottom variant='headline'>
                  Irrigation Controller
                </Typography>
                <Typography>
                  Last updated: <Moment fromNow>{this.state.data.created}</Moment>
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          {items}
        </Grid>
      </main>)
    } else {
      return <LinearProgress />
    }
  }
}

export default withStyles(styles)(App)
