import React from 'react'
import Grid from '@material-ui/core/Grid'
import Plant from './Plant'
import LinearProgress from '@material-ui/core/LinearProgress'
import Error from '@material-ui/icons/Error'
import Typography from '@material-ui/core/Typography'
import Card from '@material-ui/core/Card'
import CardContent from '@material-ui/core/CardContent'
import { withStyles } from '@material-ui/core/styles'
import backgroundImage from './background.jpg'
import Moment from 'react-moment'
import { sortBy } from 'lodash/collection'

const styles = theme => ({
  root: {
    width: '100%',
    height: '100%',
    backgroundImage: `url(${backgroundImage.src})`,
    backgroundSize: 'cover',
    paddingTop: '1em',
    display: 'flex',
    justifyContent: 'center',
    alignItems: 'flex-start',
    overflow: 'auto'
  },
  grid: {
    maxWidth: '960px',
    flexWrap: 'wrap'
  },
  gridItem: {
    flexBasis: 0,
    flex: 1
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

  componentDidCatch (error, info) {
    console.log(error)
  }

  render () {
    const {classes} = this.props
    if (this.state.error) {
      return (<main className={classes.root}>
        <Grid className={classes.grid} container justify='center' spacing={32}>
          <Grid className={classes.gridItem} item xs={12}>
            <Card raised>
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
      let items = this.state.data.modules.map(module => (<Grid key={module.id} className={classes.gridItem} item>
        <Plant module={module} />
      </Grid>))
      return (<main className={classes.root}>
        <Grid className={classes.grid} container spacing={32}>
          <Grid item xs={12}>
            <Card raised>
              <CardContent>
                <Typography gutterBottom variant='headline'>
                  Irrigation Controller
                </Typography>
                <Typography>
                  Last updated: <Moment fromNow>{this.state.data.created}</Moment>
                </Typography>
                <Typography component={({children, ...props}) => (<p {...props} style={{
                  display: 'inline'
                }}>{children}</p>)}>{this.state.data.temperature.toFixed(2)}°C&nbsp;ambient</Typography>
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
