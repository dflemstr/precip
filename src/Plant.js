import React from 'react'
import { withTheme } from '@material-ui/core/styles/index'
import Card from '@material-ui/core/Card'
import CardHeader from '@material-ui/core/CardHeader'
import CardContent from '@material-ui/core/CardContent'
import CardActions from '@material-ui/core/CardActions'
import FormControlLabel from '@material-ui/core/FormControlLabel'
import FormGroup from '@material-ui/core/FormGroup'
import { AreaSeries, Crosshair, XAxis, XYPlot } from 'react-vis'
import { default as chroma } from 'chroma-js'
import 'react-vis/dist/style.css'
import Switch from '@material-ui/core/Switch'
import Typography from '@material-ui/core/Typography'
import CircularProgress from '@material-ui/core/CircularProgress'
import Power from '@material-ui/icons/Power'
import PropTypes from 'prop-types'

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
    const d = this.props.historicalHumidity[index]
    const x = new Date(d.measurement_start)
    this.setState({
      ...this.state,
      crosshairValues: [{x, y: d.min}, {x, y: d.p25}, {x, y: d.p50}, {x, y: d.p75}, {x, y: d.max}]
    })
  }

  _onMouseLeave () {
    this.setState({...this.state, crosshairValues: []})
  }

  render () {
    const {title, subtitle, theme, historicalHumidity, ...props} = this.props

    const tickColor = theme.palette.grey['500']
    const colorBase = theme.palette.primary.light
    const colorRange = [
      colorBase,
      chroma(colorBase).brighten().hex()
    ]

    const crosshairValues = this.state.crosshairValues

    const data = historicalHumidity
    let plot = null

    if (data) {
      const mins = data.map(d => ({x: new Date(d.measurement_start), y: d.min}))
      const p25s = data.map(d => ({x: new Date(d.measurement_start), y: d.p25}))
      const p50s = data.map(d => ({x: new Date(d.measurement_start), y: d.p50}))
      const p75s = data.map(d => ({x: new Date(d.measurement_start), y: d.p75}))
      const maxs = data.map(d => ({x: new Date(d.measurement_start), y: d.max}))
      plot = (<XYPlot
        width={400}
        height={100}
        xType='time-utc'
        yDomain={[0, 1.2]}
        onMouseLeave={this._onMouseLeave}
        colorType='linear'
        colorDomain={[0, 1, 2, 3, 4]}
        colorRange={colorRange}
        margin={{left: 0, right: 0, top: 0, bottom: 40}}>
        <AreaSeries
          data={maxs}
          curve='curveMonotoneX'
          color={4} />
        <AreaSeries
          data={p75s}
          onNearestX={this._onNearestX}
          curve='curveMonotoneX'
          color={3} />
        <AreaSeries
          data={p50s}
          curve='curveMonotoneX'
          color={2} />
        <AreaSeries
          data={p25s}
          curve='curveMonotoneX'
          color={1} />
        <AreaSeries
          data={mins}
          curve='curveMonotoneX'
          color={0} />
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
            <div className='rv-crosshair__item'>
              max: {Math.round(crosshairValues[4].y * 1000) / 10}%
            </div>
            <div className='rv-crosshair__item'>
              p75: {Math.round(crosshairValues[3].y * 1000) / 10}%
            </div>
            <div className='rv-crosshair__item'>
              p50: {Math.round(crosshairValues[2].y * 1000) / 10}%
            </div>
            <div className='rv-crosshair__item'>
              p25: {Math.round(crosshairValues[1].y * 1000) / 10}%
            </div>
            <div className='rv-crosshair__item'>
              min: {Math.round(crosshairValues[0].y * 1000) / 10}%
            </div>
          </div>
        </Crosshair>
        }
      </XYPlot>)
    }

    return (<Card {...props}>
      <CardHeader title={title} subtitle={subtitle} />
      <CardContent>
        {plot}

        <CircularProgress
          size={20}
          variant='static'
          value={80.3}
          style={{display: 'inline-box', verticalAlign: 'middle'}} />
        <Typography component={({children, ...props}) => (<p {...props} style={{
          display: 'inline'
        }}>{children}</p>)}>&nbsp;80.3%&nbsp;humidity</Typography>

        <CircularProgress
          size={20}
          variant='static'
          value={54.3}
          style={{display: 'inline-box', verticalAlign: 'middle', marginLeft: '8px'}} />
        <Typography component={({children, ...props}) => (<p {...props} style={{
          display: 'inline'
        }}>{children}</p>)}>&nbsp;22.3°C&nbsp;ambient</Typography>

        <Power style={{verticalAlign: 'middle', marginLeft: '8px'}} />
        <Typography component={({children, ...props}) => (<p {...props} style={{
          display: 'inline'
        }}>{children}</p>)}>Pump running</Typography>
      </CardContent>
      <CardActions>
        <FormGroup row>
          <FormControlLabel
            control={<Switch />}
            label='Force irrigation' />
        </FormGroup>
      </CardActions>
    </Card>)
  }
}

Plant.propTypes = {
  title: PropTypes.string,
  subtitle: PropTypes.string,
  historicalHumidity: PropTypes.arrayOf(PropTypes.shape({
    measurement_start: PropTypes.string.required,
    min: PropTypes.number.required,
    max: PropTypes.number.required,
    p25: PropTypes.number.required,
    p50: PropTypes.number.required,
    p75: PropTypes.number.required
  }))
}

export default withTheme()(Plant)
