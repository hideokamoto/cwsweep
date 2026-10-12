---
title: Scope
description: The AWS partitions supported by cwsweep v1.
sidebar:
  order: 2
---

v1 supports only the AWS standard partition (`aws`, commercial regions).

Because STS is a global service, the STS client used for identity verification (`sts:get-caller-identity`) is pinned to `us-east-1`. Within the commercial partition, it works regardless of which `--regions` you specify.

Behavior against member accounts in non-commercial partitions, such as AWS GovCloud (US) and China regions (`aws-us-gov` / `aws-cn` partitions), is out of scope for v1 and has not been tested.
