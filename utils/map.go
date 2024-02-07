package utils

type Hashable interface {
	HashCode() uint64
	EqualI(Hashable) bool
}

type Map map[uint64][]mapEntry

type mapEntry struct {
	e Hashable
	v interface{}
}

func (m Map) Find(e Hashable) (interface{}, bool) {
	s, ok := m[e.HashCode()]
	if !ok {
		return nil, false
	}
	for _, x := range s {
		if x.e.EqualI(e) {
			return x.v, true
		}
	}
	return nil, false
}

func (m Map) Set(e Hashable, v interface{}) {
	h := e.HashCode()
	s, ok := m[h]
	if !ok {
		s = make([]mapEntry, 0, 1)
	} else {
		for _, x := range s {
			if x.e.EqualI(e) {
				x.v = v
				return
			}
		}
	}
	m[h] = append(s, mapEntry{
		e: e,
		v: v,
	})
}

// when exists, do nothing
func (m Map) Add(e Hashable, v interface{}) interface{} {
	h := e.HashCode()
	s, ok := m[h]
	if !ok {
		s = make([]mapEntry, 0, 1)
	} else {
		for _, x := range s {
			if x.e.EqualI(e) {
				return x.v
			}
		}
	}
	m[h] = append(s, mapEntry{
		e: e,
		v: v,
	})
	return v
}

func (m Map) FilterKeys(f func(interface{}) bool) []Hashable {
	keys := []Hashable{}
	for _, s := range m {
		for _, x := range s {
			if f(x.v) {
				keys = append(keys, x.e)
			}
		}
	}
	return keys
}

func (m Map) Clear() {
	for k := range m {
		delete(m, k)
	}
}
