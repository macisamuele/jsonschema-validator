[WIP] JsonSchema Validator
==========================

***This repository is meant as personal exercise for learning [Rust](https://www.rust-lang.org/).***

<!-- [![Linux Build on CircleCI](https://circleci.com/gh/macisamuele/TODO/tree/master.svg?style=shield)](https://circleci.com/gh/macisamuele/TODO/tree/master) -->
[![Linux Build on TravisCI](https://img.shields.io/travis/com/macisamuele/jsonschema-validator/master.svg?logo=travis&label=Linux)](https://travis-ci.com/macisamuele/jsonschema-validator)
[![Windows Build on AppVeyor](https://img.shields.io/appveyor/ci/macisamuele/jsonschema-validator/master.svg?logo=appveyor&label=Windows)](https://ci.appveyor.com/project/macisamuele/jsonschema-validator)
[![Coverage](https://img.shields.io/codecov/c/github/macisamuele/jsonschema-validator/master.svg)](https://codecov.io/gh/macisamuele/jsonschema-validator)

Rationale
---------
At the moment Rust does not have any really good JsonSchema validation library. The majority does provide few
validation features and none of them is defined in a way that will provide multiple JsonSchema draft support.
An other aspect lacking on the libraries found around is that validation necessarily reports all the validation
errors, but if we're interesting on the binary answer for _is this representation valid?_ then having 1 or 100
errors does not change the answer but just the execution time.
The idea of this library is to set up a clean environment that allows:
 * simple API interaction (this does not mean that the result should be feature empty)
 * multiple draft support (draft4 will be implemented first)
 * validation errors should be returned in iterables (so we can accomodate the usecase of the binary question as well as getting all the reasons for an object not being recognized as valid)

Contributing
------------
The project is not implemented yet and a lot of features are missing, so please keep it in mind while opening Issues or Pull Requests.

ℹ️ issues requiring features will be appreciated but I would not guarantee that those will be implemented on the first iteration.
