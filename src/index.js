import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import registerServiceWorker from './registerServiceWorker';
import { MuiThemeProvider } from "@material-ui/core/styles";
import CssBaseline from "@material-ui/core/CssBaseline";
import { createMuiTheme } from "@material-ui/core/styles/index";
import 'typeface-roboto'

const theme = createMuiTheme({});

ReactDOM.render(
  <MuiThemeProvider theme={theme}>
    <CssBaseline/>
    <App/>
  </MuiThemeProvider>, document.getElementById('root'));

registerServiceWorker();
