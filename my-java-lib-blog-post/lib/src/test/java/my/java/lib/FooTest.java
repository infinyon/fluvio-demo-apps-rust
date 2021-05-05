package my.java.lib;

import org.junit.Test;
import static org.junit.Assert.*;
import my.java.lib.Foo;

public class FooTest {
    @Test public void testSomeLibraryMethod() {
        Foo foo = new Foo(10);
        assertTrue("Foo.val", foo.val() == 10);
        foo.set_field(15);
        assertTrue("Foo.val", foo.val() == 15);
    }
}
