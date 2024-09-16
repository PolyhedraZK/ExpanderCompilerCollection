package utils

// pad to 2^n gates (and 4^n for first layer)
// 4^n exists for historical reasons, not used now
func NextPowerOfTwo(x int, is4 bool) int {
	padk := 0
	for x > (1 << padk) {
		padk++
	}
	if is4 && padk%2 != 0 {
		padk++
	}
	return 1 << padk
}
