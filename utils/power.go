package utils

func NextPowerOfTwo(x int, is4 bool) int {
	// compute pad to 2^n gates (and 4^n for first layer)
	// and n>=1
	padk := 1
	for x > (1 << padk) {
		padk++
	}
	if is4 && padk%2 != 0 {
		padk++
	}
	return 1 << padk
}
