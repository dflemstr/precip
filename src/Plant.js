import React from 'react'
import { withStyles } from '@material-ui/core/styles'
import Card from '@material-ui/core/Card'
import CardHeader from '@material-ui/core/CardHeader'
import CardContent from '@material-ui/core/CardContent'
import {
  AreaSeries, Crosshair, XAxis, VerticalRectSeries, FlexibleWidthXYPlot
} from 'react-vis'
import { default as chroma } from 'chroma-js'
import 'react-vis/dist/style.css'
import Typography from '@material-ui/core/Typography'
import CircularProgress from '@material-ui/core/CircularProgress'
import Power from '@material-ui/icons/Power'
import PropTypes from 'prop-types'
import Grid from '@material-ui/core/Grid'
import { min, max } from 'lodash'
import ReactMarkdown from 'react-markdown'

const styles = theme => ({
  card: {
    minWidth: 275,
    maxWidth: 448
  }
})

class Plant extends React.Component {
  constructor (props) {
    super(props)

    this.state = {
      crosshairValues: []
    }

    this._onMouseLeave = this._onMouseLeave.bind(this)
    this._onNearestX = this._onNearestX.bind(this)
  }

  _onNearestX (value, {index}) {
    const data = this.props.module.moistureTimeseries
    const x = new Date(data.measurementStart[index])
    this.setState({
      ...this.state,
      crosshairValues: [
        {x, y: data.min[index]},
        {x, y: data.p25[index]},
        {x, y: data.p50[index]},
        {x, y: data.p75[index]},
        {x, y: data.max[index]}
      ]
    })
  }

  _onMouseLeave () {
    this.setState({...this.state, crosshairValues: []})
  }

  render () {
    const {theme, classes, module, ...props} = this.props

    const tickColor = theme.palette.grey['500']
    const colorBase = theme.palette.primary.light
    const colorRange = [
      colorBase,
      chroma(colorBase).brighten().brighten().hex(),
      '#ff0000'
    ]
    const colorMark = chroma(theme.palette.secondary.light).alpha(0.5).css()
    const legendTextColor = theme.palette.common.white

    const crosshairValues = this.state.crosshairValues

    let moistureRatio = module.lastMoisture ? (module.lastMoisture - module.minMoisture) / (module.maxMoisture - module.minMoisture) : null
    let moisturePadding = (module.maxMoisture - module.minMoisture) * 0.1

    let plot = null

    if (module.moistureTimeseries) {
      const data = module.moistureTimeseries
      const xs = data.measurementStart.map(v => new Date(v))
      const mins = data.min.map((v, i) => ({x: xs[i], y: v}))
      const p25s = data.p25.map((v, i) => ({x: xs[i], y: v}))
      const p50s = data.p50.map((v, i) => ({x: xs[i], y: v}))
      const p75s = data.p75.map((v, i) => ({x: xs[i], y: v}))
      const maxs = data.max.map((v, i) => ({x: xs[i], y: v}))
      const start = min(xs)
      const end = max(xs)
      const yAxisMin = Math.min(module.minMoisture - moisturePadding, module.targetMinMoisture - moisturePadding)
      const yAxisMax = Math.max(module.maxMoisture + moisturePadding, module.targetMaxMoisture + moisturePadding)
      const pumpings = module.pumpRunning.map(v => ({x: new Date(v[0]), x0: new Date(v[1]), y: yAxisMin, y0: yAxisMax}))

      plot = (
        <FlexibleWidthXYPlot
          height={100}
          xType='time-utc'
          yDomain={[yAxisMin, yAxisMax]}
          onMouseLeave={this._onMouseLeave}
          colorType='linear'
          colorDomain={[0.0, 2.0]}
          colorRange={colorRange}
          margin={{left: 0, right: 0, top: 0, bottom: 40}}>
          <AreaSeries
            data={maxs}
            curve='curveMonotoneX'
            color={1.0} />
          <AreaSeries
            data={p75s}
            onNearestX={this._onNearestX}
            curve='curveMonotoneX'
            color={0.75} />
          <AreaSeries
            data={p50s}
            curve='curveMonotoneX'
            color={0.5} />
          <AreaSeries
            data={p25s}
            curve='curveMonotoneX'
            color={0.25} />
          <AreaSeries
            data={mins}
            curve='curveMonotoneX'
            color={0} />
          <VerticalRectSeries
            data={pumpings}
            colorType='literal'
            color={colorMark}
          />
          <VerticalRectSeries
            data={[{x: end, x0: start, y: module.targetMinMoisture, y0: module.targetMaxMoisture}]}
            colorType='literal'
            color='rgba(0, 255, 0, 0.1)'
          />
          <XAxis
            tickTotal={6}
            tickSizeInner={0}
            style={{
              line: {
                stroke: 'none'
              },
              ticks: {
                fill: tickColor
              },
              text: {
                fontFamily: theme.typography.body1.fontFamily,
                fontSize: theme.typography.body1.fontSize
              }
            }} />
          {crosshairValues.length > 0 &&
          <Crosshair values={crosshairValues}>
            <div className='rv-crosshair__inner__content'>
              <Typography className='rv-crosshair__item' variant='caption' style={{color: legendTextColor}}>
                max&#9;{Math.round(crosshairValues[4].y * 1000) / 1000}<br />
                p75&#9;{Math.round(crosshairValues[3].y * 1000) / 1000}<br />
                p50&#9;{Math.round(crosshairValues[2].y * 1000) / 1000}<br />
                p25&#9;{Math.round(crosshairValues[1].y * 1000) / 1000}<br />
                min&#9;{Math.round(crosshairValues[0].y * 1000) / 1000}
              </Typography>
            </div>
          </Crosshair>
          }
        </FlexibleWidthXYPlot>)
    }

    return (<Card className={classes.card} raised {...props}>
      <CardHeader title={module.name} subheader={<ReactMarkdown source={module.description} />} />
      <CardContent>
        {plot}

        <Grid container spacing={8}>
          {moistureRatio && <Grid item>
            <CircularProgress
              size={20}
              variant='static'
              value={moistureRatio * 100}
              style={{display: 'inline-box', verticalAlign: 'middle', marginRight: '8px'}} />
            <Typography component={({children, ...props}) => (<p {...props} style={{
              display: 'inline'
            }}>{children}</p>)}>{Math.round(moistureRatio * 1000) / 10}%&nbsp;moisture</Typography>
          </Grid>}
          <Grid item>
            <CircularProgress
              size={20}
              variant='static'
              value={54.3}
              style={{display: 'inline-box', verticalAlign: 'middle', marginRight: '8px'}} />
            <Typography component={({children, ...props}) => (<p {...props} style={{
              display: 'inline'
            }}>{children}</p>)}>22.3°C&nbsp;ambient</Typography>
          </Grid>
          <Grid item>
            <Power style={{verticalAlign: 'middle'}} />
            <Typography component={({children, ...props}) => (<p {...props} style={{
              display: 'inline'
            }}>{children}</p>)}>Pump running</Typography>
          </Grid>
        </Grid>
      </CardContent>
    </Card>)
  }
}

Plant.propTypes = {
  module: PropTypes.shape({
    minMoisture: PropTypes.number.isRequired,
    maxMoisture: PropTypes.number.isRequired,
    lastMoisture: PropTypes.number.isRequired,
    moistureTimeseries: PropTypes.shape({
      measurementStart: PropTypes.arrayOf(PropTypes.string),
      min: PropTypes.arrayOf(PropTypes.number),
      max: PropTypes.arrayOf(PropTypes.number),
      p25: PropTypes.arrayOf(PropTypes.number),
      p50: PropTypes.arrayOf(PropTypes.number),
      p75: PropTypes.arrayOf(PropTypes.number)
    }).isRequired
  })
}

export default withStyles(styles, {withTheme: true})(Plant)
