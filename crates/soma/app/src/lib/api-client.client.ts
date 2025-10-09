"use client";
import createClient from "openapi-react-query";
import fetchClient from "./api-client";

const $api = createClient(fetchClient);

export default $api;
