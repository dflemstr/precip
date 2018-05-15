import React from 'react'
import ReactDOM from 'react-dom'
import App from './App'

/* global beforeEach expect fetch it */

beforeEach(() => {
  fetch.resetMocks()
})

it('renders without crashing', () => {
  const div = document.createElement('div')
  fetch.mockResponseOnce(JSON.stringify({
    'modules': [{
      'id': '71f3abbf-eda2-4b1f-a9a2-8af7f290e0a6',
      'name': 'Plant 2',
      'running': false,
      'force_running': false,
      'historical_humidity': [{
        'measurement_start': '2018-05-15T17:05:00Z',
        'min': 0.7440120858660881,
        'max': 0.8178992436390712,
        'p25': 0.7769198054604457,
        'p50': 0.7913926075090871,
        'p75': 0.807740966624666
      }]
    }]
  }))
  ReactDOM.render(<App />, div)
  ReactDOM.unmountComponentAtNode(div)
  expect(fetch.mock.calls[0][0]).toEqual('https://s3-eu-west-1.amazonaws.com/precip-stats/data.json')
})
