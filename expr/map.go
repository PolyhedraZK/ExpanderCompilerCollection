package expr

type Map map[uint64][]mapEntry

type mapEntry struct {
	e Expression
	v interface{}
}

func (m Map) Find(e Expression) (interface{}, bool) {
	s, ok := m[e.HashCode()]
	if !ok {
		return nil, false
	}
	for _, x := range s {
		if x.e.Equal(e) {
			return x.v, true
		}
	}
	return nil, false
}

func (m Map) Set(e Expression, v interface{}) {
	h := e.HashCode()
	s, ok := m[h]
	if !ok {
		s = make([]mapEntry, 0, 1)
	} else {
		for _, x := range s {
			if x.e.Equal(e) {
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
func (m Map) Add(e Expression, v interface{}) {
	h := e.HashCode()
	s, ok := m[h]
	if !ok {
		s = make([]mapEntry, 0, 1)
	} else {
		for _, x := range s {
			if x.e.Equal(e) {
				return
			}
		}
	}
	m[h] = append(s, mapEntry{
		e: e,
		v: v,
	})
}

func (m Map) FilterKeys(f func(interface{}) bool) []Expression {
	keys := []Expression{}
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
