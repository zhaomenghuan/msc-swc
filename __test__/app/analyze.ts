import { Module, ModuleTypes } from './module';
import { log } from './utils/util';

class AppModule extends Module {
  build() {
    log(`AppModule build, id: ${this.id}, module_type: ${this.moduleType}`);
    const pageModule = createModule('/page', ModuleTypes.Page);
    this.addDependency(pageModule.getId());
    pageModule.build();
  }
}

class PageModule extends Module {
  build() {
    log(`PageModule build, id: ${this.id}, module_type: ${this.moduleType}`);
    const componentModule = createModule('/component', ModuleTypes.Component);
    this.addDependency(componentModule.getId());
    componentModule.build();
  }
}

class ComponentModule extends Module {
  build() {
    log(`ComponentModule build, id: ${this.id}, module_type: ${this.moduleType}`);
  }
}

function createModule(id: string, moduleType: ModuleTypes): Module {
  let module: Module;
  switch (moduleType) {
    case ModuleTypes.App: {
      module = new AppModule(id, ModuleTypes.App);
      break;
    }

    case ModuleTypes.Page: {
      module = new PageModule(id, ModuleTypes.Page);
      break;
    }

    case ModuleTypes.Component: {
      module = new ComponentModule(id, ModuleTypes.Component);
      break;
    }

    default: {
      throw new Error(`Invalid module type: ${moduleType}`);
    }
  }

  return module;
}

const appModule = createModule('/app', ModuleTypes.App);
appModule.build();
