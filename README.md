# autosar-data-abstraction

[![Github Actions](https://github.com/DanielT/autosar-data-abstraction/actions/workflows/CI.yml/badge.svg)](https://github.com/DanielT/autosar-data-abstraction/actions)

This is an abstraction layer on top of the autosar-data model.

Rather than transforming the element based model into a new form, it only presents a view into the existing model, and provides methods to retrieve and modify the data.

As a result the use of autosar-data-abstraction can be freely mixed with direct manipulation of the model. Allowing the two to be mixed freely is a very important design goal: autosar-data-abstraction only represents a tiny portion of all possible elements in an Autosar file.
