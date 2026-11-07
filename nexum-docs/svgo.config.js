export default {
	multipass: true,
	// SVGO 4 dropped removeViewBox from preset-default and preserves viewBox by
	// default — exactly what the logo pipeline requires. Keeping the explicit
	// preset name documents that intent; no override is needed or allowed.
	plugins: [{ name: 'preset-default' }],
};
