"use server";

import { serialize } from "next-mdx-remote/serialize";

export async function getSerializedMdx(source: string) {
	const mdxSource = await serialize(source);
	return mdxSource;
}
