
export const StateEnum = {
  detail: 1,
  searchResult: 2,
  mainPage: 3
}
export type StateEnum = (typeof StateEnum)[keyof typeof StateEnum];
export interface ContentState {
  currentState: StateEnum,
  setCurrentState: (state: StateEnum) => void
}
