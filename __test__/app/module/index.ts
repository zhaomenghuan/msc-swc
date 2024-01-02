export enum ModuleTypes {
  App = 'App',
  Page = 'Page',
  Component = 'Component',
}

export abstract class Module {
  protected id: string;
  protected moduleType: ModuleTypes;
  protected dependencies: Set<string>;

  constructor(id: string, moduleType: ModuleTypes) {
    this.id = id;
    this.moduleType = moduleType;
    this.dependencies = new Set();
  }

  getId() {
    return this.id;
  }

  getModuleType() {
    return this.moduleType;
  }

  addDependency(id: string) {
    this.dependencies.add(id);
  }

  abstract build(): void;
}
