# id3-rs

[![Build Status](https://travis-ci.com/schneefux/node-id3-rs.svg?branch=master)](https://travis-ci.com/schneefux/node-id3-rs)
[![NPM Version](https://img.shields.io/npm/v/id3-rs.svg?sanitize=true)](https://www.npmjs.com/package/id3-rs)

Bindings for [rust-id3](https://github.com/polyfloyd/rust-id3).

Note: This library is not production-ready.
For an alternative with support for more formats, see [taglib3](https://github.com/schneefux/node-taglib3). 

## Installation

See the [Neon Bindings docs](https://neon-bindings.com/docs/getting-started/).

## Usage

At the moment, the data structures are very low-level. Binary data is encoded in base64.

### Reading tags

```js
const id3 = require('id3-rs')
// sync
console.log(id3.readTagsSync('file.mp3'))
// async
id3.readTags('file.mp3', (error, data) => console.log(error, data))
```

```json
[
  { "id": "TIT2", "text": "Hope" },
  { "id": "RVAD", "data": "ABAAAAAAAAAAAA==" }
]
```

### Writing tags

The method `replaceTagAtIndexSync` can be used to replace a frame (index as per `readTags`). If the index is out of bounds, the frame is appended.

```js
const id3 = require('id3-rs')
// sync
id3.replaceTagAtIndexSync('file.mp3', -1, 'TBPM', '83')
// async
id3.replaceTagAtIndex('file.mp3', -1, 'TBPM', '83', (error, data) => console.log(error, data))
```
