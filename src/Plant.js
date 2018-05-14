import React from 'react';
import { withTheme } from "@material-ui/core/styles/index";
import Card from '@material-ui/core/Card';
import CardHeader from '@material-ui/core/CardHeader';
import CardContent from '@material-ui/core/CardContent';
import CardActions from '@material-ui/core/CardActions';
import FormControlLabel from "@material-ui/core/FormControlLabel";
import FormGroup from "@material-ui/core/FormGroup";
import { AreaSeries, Crosshair, XAxis, XYPlot } from 'react-vis';
import * as chroma from "chroma-js";
import "react-vis/dist/style.css";
import Switch from "@material-ui/core/Switch";
import Typography from "@material-ui/core/Typography";
import CircularProgress from "@material-ui/core/CircularProgress";
import Power from "@material-ui/icons/Power";

const data1 = [
  {x: (1526318866 + 1 * 60 * 60) * 1000, y: 0.8},
  {x: (1526318866 + 2 * 60 * 60) * 1000, y: 0.6},
  {x: (1526318866 + 3 * 60 * 60) * 1000, y: 0.8},
  {x: (1526318866 + 4 * 60 * 60) * 1000, y: 1.0},
  {x: (1526318866 + 5 * 60 * 60) * 1000, y: 0.9},
  {x: (1526318866 + 6 * 60 * 60) * 1000, y: 0.7},
  {x: (1526318866 + 7 * 60 * 60) * 1000, y: 0.8},
  {x: (1526318866 + 8 * 60 * 60) * 1000, y: 0.6},
];

const data2 = data1.map(({x, y}) => ({x, y: y * 0.8}));
const data3 = data1.map(({x, y}) => ({x, y: y * 0.4}));

const data = [data1, data2, data3];

class Plant extends React.Component {
  constructor(props) {
    super(props);

    this.state = {
      crosshairValues: []
    };

    this._onMouseLeave = this._onMouseLeave.bind(this);
    this._onNearestX = this._onNearestX.bind(this);
  }

  _onNearestX(value, {index}) {
    this.setState({...this.state, crosshairValues: data.map(d => d[index])});
  }

  _onMouseLeave() {
    this.setState({...this.state, crosshairValues: []});
  }

  render() {
    const {title, subtitle, theme} = this.props;

    const tickColor = theme.palette.grey['500'];
    const colorBase = theme.palette.primary.light;
    const colorRange = [
      colorBase,
      chroma(colorBase).brighten().hex()
    ];

    const crosshairValues = this.state.crosshairValues;

    return (<Card>
      <CardHeader title={title} subtitle={subtitle}/>
      <CardContent>
        <XYPlot
          width={400}
          height={100}
          xType="time-utc"
          yDomain={[0, 1]}
          onMouseLeave={this._onMouseLeave}
          colorType="linear"
          colorDomain={[0, 1, 2]}
          colorRange={colorRange}
          margin={{left: 0, right: 0, top: 0, bottom: 40}}>
          <AreaSeries
            data={data1}
            onNearestX={this._onNearestX}
            curve="curveMonotoneX"
            color={2}/>
          <AreaSeries
            data={data2}
            curve="curveMonotoneX"
            color={1}/>
          <AreaSeries
            data={data3}
            curve="curveMonotoneX"
            color={0}/>
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
            }}/>
          {crosshairValues.length > 0 &&
           <Crosshair values={crosshairValues}>
             <div className="rv-crosshair__inner__content">
               <div className="rv-crosshair__item">
                 min: {Math.round(crosshairValues[0].y * 1000) / 10}%
               </div>
               <div className="rv-crosshair__item">
                 avg: {Math.round(crosshairValues[1].y * 1000) / 10}%
               </div>
               <div className="rv-crosshair__item">
                 max: {Math.round(crosshairValues[2].y * 1000) / 10}%
               </div>
             </div>
           </Crosshair>
          }
        </XYPlot>

        <CircularProgress
          size={20}
          variant="static"
          value={80.3}
          style={{display: 'inline-box', verticalAlign: 'middle'}}/>
        <Typography component={({children, ...props}) => (<p {...props} style={{
          display: "inline"
        }}>{children}</p>)}>&nbsp;80.3%&nbsp;moisture</Typography>

        <CircularProgress
          size={20}
          variant="static"
          value={54.3}
          style={{display: 'inline-box', verticalAlign: 'middle', marginLeft: '8px'}}/>
        <Typography component={({children, ...props}) => (<p {...props} style={{
          display: "inline"
        }}>{children}</p>)}>&nbsp;22.3°C&nbsp;ambient</Typography>

        <Power style={{verticalAlign: 'middle', marginLeft: '8px'}}/>
        <Typography component={({children, ...props}) => (<p {...props} style={{
          display: "inline"
        }}>{children}</p>)}>Pump running</Typography>
      </CardContent>
      <CardActions>
        <FormGroup row>
          <FormControlLabel
            control={<Switch/>}
            label="Force irrigation"/>
        </FormGroup>
      </CardActions>
    </Card>);
  }
}

export default withTheme()(Plant);
