package helper;

import javax.imageio.*;
import java.awt.image.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

public class Helper {
	public static final Path root = Paths.get("../../");
	public static final Path models = root.resolve("src").resolve("res");
	public static final Path cCode = root.resolve("src").resolve("c");
	public static final Path rustRes = root.resolve("src").resolve("rust_casted").resolve("res");

	public static void main(String[] args) throws IOException {
		// var obj = parseOBJ(readFile(models.resolve("cube.obj"))).serialize();
		byte[] obj = parseOBJ(readFile(models.resolve("pb/rpan.obj"))).serialize();
		byte[] texture = parsePNG(readFile(models.resolve("pb/rpan_hr.png"))).serialize();
		byte[] made_with = parsePNG(readFile(models.resolve("made_with.png"))).serialize();
		// byte[] cArray = toCArray("model_bin", sobj);
		// writeFile(cCode.resolve("model_bin.c"), cArray);
		writeFile(rustRes.resolve("model.rust3d"), obj);
		writeFile(rustRes.resolve("texture.rust2d"), texture);
		writeFile(rustRes.resolve("made_with.rust2d"), made_with);
	}

	private static void writeFile(Path path, byte[] binary) throws IOException {
		Files.write(path, binary, StandardOpenOption.CREATE, StandardOpenOption.TRUNCATE_EXISTING, StandardOpenOption.WRITE);
	}

	private static byte[] toCArray(final String variable, byte[] binary) {
		StringBuilder sb = new StringBuilder();
		sb.append("const unsigned char "+variable+"[] = {");
		boolean needsComma = false;
		for(byte octet : binary) {
			if(needsComma)
				sb.append(",");
			else
				needsComma = true;

			sb.append(octet);
		}
		sb.append("};");

		return sb.toString().getBytes();
	}

	private static OBJ parseOBJ(byte[] raw) {
		String s = new String(raw);
		OBJ obj = new OBJ();
		for(String line : s.split("\n")) {
			if(line.startsWith("#"))
				continue;

			for(int i = 16; i != 0; i--) {
				line = line.replace("  ", " ");
				line = line.replace("\t", " ");
			}

			String[] els = line.split(" ");
			if(els.length < 2)
				continue;

			switch(els[0]) {
				case "v":
					Vertex vertex = new Vertex(Float.parseFloat(els[1]), Float.parseFloat(els[2]), Float.parseFloat(els[3]));
					obj.vertices.add(vertex);
					break;
				case "vt":
					Vertex2D texture_vertex = new Vertex2D(Float.parseFloat(els[1]), 1 - Float.parseFloat(els[2]));
					obj.texture_vertices.add(texture_vertex);
					break;
				case "f":
					int[] vecs = new int[3];
					int[] vt = new int[3];
					for(int i = 0; i != 3; i++) {
						String[] parts = els[1+i].split("/");
						vecs[i] = Integer.parseUnsignedInt(parts[0])-1;
						vt[i] = Integer.parseUnsignedInt(parts[1])-1;
					}
					Triangle triangle = new Triangle(vecs[0], vecs[1], vecs[2], vt[0], vt[1], vt[2]);
					obj.triangles.add(triangle);
					break;
			}
		}
		return obj;
	}

	private static Texture parsePNG(byte[] binary) throws IOException {
		BufferedImage img = ImageIO.read(new ByteArrayInputStream(binary));
		short w = (short)img.getWidth();
		short h = (short)img.getHeight();
		int[] data = new int[w*h];
		byte[] converted = new byte[data.length * 4];
		img.getRGB(0,0,w,h,data,0,w);
		for(int y = 0; y != h; y++) {
			for(int x = 0; x != h; x++) {
				int px = data[y*w+x];
				int a = 0xFF & (px >> 24);
				int r = 0xFF & (px >> 16);
				int g = 0xFF & (px >>  8);
				int b = 0xFF & (px      );

				converted[4*(y*w+x)  ] = (byte) r;
				converted[4*(y*w+x)+1] = (byte) g;
				converted[4*(y*w+x)+2] = (byte) b;
				converted[4*(y*w+x)+3] = (byte) a;
			}
		}
		return new Texture(w, h, converted);
	}

	private static byte[] readFile(Path path) throws IOException {
		return Files.readAllBytes(path);
	}

	static class Vertex2D {
		float x, y;

		public Vertex2D(float x, float y) {
			this.x = x;
			this.y = y;
		}
	}

	static class Vertex {
		float x, y, z;

		public Vertex(float x, float y, float z) {
			this.x = x;
			this.y = y;
			this.z = z;
		}
	}

	static class Triangle {
		int vertices[];
		int texture_vertices[];

		public Triangle(int v1, int v2, int v3, int vt1, int vt2, int vt3) {
			vertices = new int[] {v1,v2,v3};
			texture_vertices = new int[] {vt1,vt2,vt3};
		}
	}

	static class OBJ {
		ArrayList<Vertex> vertices = new ArrayList<>();
		ArrayList<Vertex2D> texture_vertices = new ArrayList<>();
		ArrayList<Triangle> triangles = new ArrayList<>();

		private void optimize() {
			for(Triangle t : triangles) {
				System.out.println(vertices.get(t.vertices[0]).x+" "+vertices.get(t.vertices[0]).y+" "+vertices.get(t.vertices[0]).z);
				System.out.println(vertices.get(t.vertices[1]).x+" "+vertices.get(t.vertices[1]).y+" "+vertices.get(t.vertices[1]).z);
				System.out.println(vertices.get(t.vertices[2]).x+" "+vertices.get(t.vertices[2]).y+" "+vertices.get(t.vertices[2]).z);
			}
		}

		byte[] serialize() throws IOException {
			this.optimize();
			ByteArrayOutputStream out = new ByteArrayOutputStream();
			DataOutputStream dos = new DataOutputStream(out);
			dos.writeShort(this.vertices.size());
			dos.writeShort(this.texture_vertices.size());
			dos.writeShort(this.triangles.size());
			for(Vertex v : this.vertices) {
				dos.writeFloat(v.x);
				dos.writeFloat(v.y);
				dos.writeFloat(v.z);
			}
			for(Vertex2D vt : this.texture_vertices) {
				dos.writeFloat(vt.x);
				dos.writeFloat(vt.y);
			}
			for(Triangle t : this.triangles) {
				dos.writeShort(t.vertices[0]);
				dos.writeShort(t.vertices[1]);
				dos.writeShort(t.vertices[2]);
				dos.writeShort(t.texture_vertices[0]);
				dos.writeShort(t.texture_vertices[1]);
				dos.writeShort(t.texture_vertices[2]);
			}
			dos.flush();
			out.flush();
			return out.toByteArray();
		}
	}

	static class Texture {
		short width, height;
		byte[] data;

		Texture(short width, short height, byte[] data) {
			this.width  = width;
			this.height = height;
			this.data = data;
		}

		byte[] serialize() throws IOException {
			ByteArrayOutputStream out = new ByteArrayOutputStream();
			DataOutputStream dos = new DataOutputStream(out);
			dos.writeShort(width);
			dos.writeShort(height);
			dos.write(data);
			dos.flush();
			out.flush();
			return out.toByteArray();
		}
	}
}