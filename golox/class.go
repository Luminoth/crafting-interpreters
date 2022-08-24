package main

type LoxClass struct {
	Name string `json:"name"`
}

func (c LoxClass) String() string {
	return c.Name
}
