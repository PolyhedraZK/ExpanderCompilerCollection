package utils

import "sort"

type IntSeq struct {
	s   []int
	cmp func(int, int) bool
}

func (l *IntSeq) Len() int {
	return len(l.s)
}

func (l *IntSeq) Swap(i, j int) {
	l.s[i], l.s[j] = l.s[j], l.s[i]
}

func (l *IntSeq) Less(i, j int) bool {
	return l.cmp(l.s[i], l.s[j])
}

func SortIntSeq(s []int, cmp func(int, int) bool) {
	l := &IntSeq{
		s:   s,
		cmp: cmp,
	}
	sort.Sort(l)
}
