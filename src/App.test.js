import React from 'react'
import App from './App'
import renderer from 'react-test-renderer'
import { JssProvider } from 'react-jss'

/* global beforeEach expect fetch it */

beforeEach(() => {
  fetch.resetMocks()
})

it('renders moisture values', (done) => {
  const data = {
    'created': '2018-05-15T17:05:00Z',
    'modules': [{
      'id': '71f3abbf-eda2-4b1f-a9a2-8af7f290e0a6',
      'name': 'Plant 2',
      'running': false,
      'forceRunning': false,
      'minMoisture': 0.25,
      'maxMoisture': 0.9,
      'lastMoisture': 0.8,
      'targetMinMoisture': 0.3,
      'targetMaxMoisture': 0.8,
      'pumpRunning': [['2018-05-15T17:05:00Z', '2018-05-15T17:05:01Z']],
      'moistureTimeseries': {
        'measurementStart': ['2018-05-15T17:05:00Z'],
        'min': [0.7440120858660881],
        'max': [0.8178992436390712],
        'p25': [0.7769198054604457],
        'p50': [0.7913926075090871],
        'p75': [0.807740966624666]
      }
    }]
  }

  fetch.mockResponseOnce(JSON.stringify(data))

  const generator = (rule, sheet) => `${sheet.options.classNamePrefix}-${rule.key}`
  const component = renderer.create(<JssProvider generateClassName={generator}><App /></JssProvider>)

  // TODO: hack
  setTimeout(() => {
    const tree = component.toJSON()
    expect(tree).toMatchSnapshot()
    expect(fetch.mock.calls[0][0]).toEqual('https://s3-eu-west-1.amazonaws.com/precip-stats/data.json')
    done()
  }, 500)
})

it('renders error', (done) => {
  fetch.mockReject(new Error('failure foo bar baz test'))

  const generator = (rule, sheet) => `${sheet.options.classNamePrefix}-${rule.key}`
  const component = renderer.create(<JssProvider generateClassName={generator}><App /></JssProvider>)

  // TODO: hack
  setTimeout(() => {
    const tree = component.toJSON()
    expect(tree).toMatchSnapshot()
    expect(fetch.mock.calls[0][0]).toEqual('https://s3-eu-west-1.amazonaws.com/precip-stats/data.json')
    done()
  }, 500)
})
