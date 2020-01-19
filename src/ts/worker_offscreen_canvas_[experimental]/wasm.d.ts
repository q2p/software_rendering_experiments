declare namespace WebAssembly {
	type Imports = Array<{
			name: string;
			kind: string;
	}>;

	type Exports = Array<{
			module: string;
			name: string;
			kind: string;
	}>;

	class Module {
		constructor(bufferSource: ArrayBuffer | Uint8Array);
		static customSections(module: Module, sectionName: string): ArrayBuffer[];
		static exports(module: Module): Imports;
		static imports(module: Module): Exports;
	}

	class Instance {
		readonly exports: any;
		constructor(module: Module, importObject?: any);
	}

	interface ResultObject {
		module: Module;
		instance: Instance;
	}

	interface MemoryDescriptor {
			initial: number;
			maximum?: number;
	}

	class Memory {
			readonly buffer: ArrayBuffer;
			constructor(memoryDescriptor: MemoryDescriptor);
			grow(numPages: number): number;
	}

	interface TableDescriptor {
			element: "anyfunc";
			initial: number;
			maximum?: number;
	}

	class Table {
			readonly length: number;
			constructor(tableDescriptor: TableDescriptor);
			get(index: number): (args: any[]) => any;
			grow(numElements: number): number;
			set(index: number, value: (args: any[]) => any): void;
	}

	class CompileError extends Error {
			toString(): string;
			readonly fileName: string;
			readonly lineNumber: string;
			readonly columnNumber: string;
			constructor(message?: string, fileName?: string, lineNumber?: number);
	}

	class LinkError extends Error {
			readonly fileName: string;
			readonly lineNumber: string;
			readonly columnNumber: string;
			constructor(message?: string, fileName?: string, lineNumber?: number);
			toString(): string;
	}

	class RuntimeError extends Error {
			readonly fileName: string;
			readonly lineNumber: string;
			readonly columnNumber: string;
			constructor(message?: string, fileName?: string, lineNumber?: number);
			toString(): string;
	}

	function compile(bufferSource: ArrayBuffer | Uint8Array):Promise<Module>;

	function instantiate(bufferSource: ArrayBuffer | Uint8Array, importObject?: object):Promise<ResultObject>;
	function instantiate(module: Module, importObject?: object):Promise<Instance>;

	function validate(bufferSource: ArrayBuffer | Uint8Array):boolean;
}